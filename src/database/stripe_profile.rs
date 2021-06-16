use crate::route::EndpointProcessingError;

use super::{
    user,
    user::{User, UserID},
    DBConnection, SqlPool, SyncTransaction,
};
use rusqlite::params;

pub type StripeID = String;

pub struct StripeSubscription {
    pub start: i64,
    pub expiration: i64,
}

pub struct StripeProfile {
    pub stripe_id: StripeID,
    pub user: UserID,
    pub subscription: Option<StripeSubscription>,
    pub last_updated: i64,
}

pub async fn create(
    user: User,
    pool: &SqlPool,
) -> Result<StripeProfile, Box<dyn std::error::Error + 'static>> {
    // We create the customer in Stripe's API
    let user_id = user.id;
    let customer_stripe_id = crate::route::payment::stripe::customer::create_customer(user).await?;

    let customer = StripeProfile {
        stripe_id: customer_stripe_id,
        user: user_id,
        subscription: None,
        last_updated: crate::helper::get_millis_since_epoch()?,
    };

    pool.get()?.execute(
        "INSERT INTO stripe_profiles
                (stripe_id, user, last_updated)
                VALUES (?1, ?2, datetime(?3, 'unixepoch'));",
        params![customer.stripe_id, user_id, customer.last_updated],
    )?;
    Ok(customer)
}

pub fn get<C: DBConnection>(
    user_id: UserID,
    con: &C,
) -> Result<Option<StripeProfile>, Box<dyn std::error::Error>> {
    let result: Result<(StripeID, Option<i64>, Option<i64>, i64), _> = con.query_row(
        "SELECT stripe_id FROM stripe_profiles WHERE user = ?1",
        params![user_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    );
    match result {
        Ok((stripe_id, expiration, start, last_updated)) => Ok(Some(StripeProfile {
            user: user_id,
            stripe_id,
            subscription: if expiration.is_some() && start.is_some() {
                Some(StripeSubscription {
                    expiration: expiration.unwrap(),
                    start: start.unwrap(),
                })
            } else {
                None
            },
            last_updated,
        })),
        Err(e) => match e.downcast_ref::<rusqlite::Error>() {
            Some(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            _ => Err(e),
        },
    }
}

pub async fn get_or_create(
    user: User,
    pool: &SqlPool,
) -> Result<StripeProfile, Box<dyn std::error::Error>> {
    // Note we have to throw away the Error since it's not sync and the await below gets angry :<
    let val = get(user.id, pool).map_err(|e| {
        println!("Got error that had to be thrown away {}", e);
        1
    });
    match val {
        Ok(Some(profile)) => Ok(profile),
        Ok(None) => create(user, pool).await,
        Err(_) => Err(Box::new(EndpointProcessingError::Unauthorized)),
    }
}

crate::basic_error!(StripeProfileNotFound, "Could not find user");

pub fn set_subscription(
    stripe_id: &StripeID,
    start: i64,
    end: i64,
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut con = pool.get()?;
    // We verify the user exists
    let user_id: UserID = match con.query_row(
        "SELECT user FROM stripe_profiles WHERE stripe_id == '?1'",
        params![stripe_id],
        |row| Ok(row.get(0)?),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(Box::new(StripeProfileNotFound));
        }
        Err(e) => return Err(Box::new(e)),
    };
    let tran = con.transaction()?;
    // Then we save the new subscription info
    tran.execute(
        "UPDATE stripe_profiles
                SET
                    stripe_subscription_start = datetime(?1, 'unixepoch'),
                    stripe_subscription_expiration = datetime(?2, 'unixepoch'),
                    last_updated = datetime('now')
                WHERE stripe_id == '?3';",
        params![start, end, stripe_id],
    )?;
    // Then we update to the newest subscription information for the user
    user::update_subscription_expiration(user_id, pool, &tran)?;
    tran.commit()?;
    Ok(())
}

pub fn remove_subscription(
    stripe_id: &StripeID,
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut con = pool.get()?;
    // We verify the user exists
    let user_id: UserID = match con.query_row(
        "SELECT user FROM stripe_profiles WHERE stripe_id == '?1'",
        params![stripe_id],
        |row| Ok(row.get(0)?),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(Box::new(StripeProfileNotFound));
        }
        Err(e) => return Err(Box::new(e)),
    };
    let tran = con.transaction()?;
    // Then we save the new subscription info
    tran.execute(
        "UPDATE stripe_profiles
                SET
                    stripe_subscription_start = NULL,
                    stripe_subscription_expiration = NULL,
                    last_updated = datetime('now')
                WHERE stripe_id == '?1';",
        params![stripe_id],
    )?;
    // Then we update to the newest subscription information for the user
    user::update_subscription_expiration(user_id, pool, &tran)?;
    tran.commit()?;
    Ok(())
}

pub async fn delete_user(
    user: &User,
    trans: &SyncTransaction,
) -> Result<bool, Box<dyn std::error::Error>> {
    // First see if this user actually exists
    let stripe_user = get(user.id, trans)?;
    let stripe_user = match stripe_user {
        Some(v) => v,
        // If there isn't any stripe user, we don't need to do anything
        None => return Ok(false),
    };

    // First we remove the user from the database (On a transaction, so not commited yet)
    trans.execute(
        "DELETE FROM stripe_profiles WHERE stripe_id = ?1",
        params![stripe_user.stripe_id],
    )?;

    // Then we try to remove them from stripe (Note if this fails, the transaction is dropped, and the stripe profile remains in the DB)
    crate::route::payment::stripe::customer::delete_customer(&stripe_user.stripe_id).await?;

    Ok(true)
}
