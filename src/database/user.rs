use super::{stripe_profile, token, DBConnection, SqlPool, SyncTransaction};
use rusqlite::{params, Transaction};
use std::convert::TryInto;

// Never negative but row.get in rusqlite does not work for u64 for some reason
pub type UserID = i64;

use crate::crypt_funcs::{MAX_SALT_SIZE, PASSWORD_HASH_SIZE};

#[derive(Clone)]
pub struct User {
    pub id: UserID,
    pub email: String,
    pub password: [u8; PASSWORD_HASH_SIZE],
    pub salt: [u8; MAX_SALT_SIZE],
    pub subscription_expiration: Option<i64>,
}

pub fn insert(
    email: &str,
    hash: &[u8],
    salt: &[u8],
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    pool.get()?.execute(
        "INSERT INTO users
                (email, password, salt)
                VALUES (?1, ?2, ?3);",
        params![email, hash, salt],
    )?;
    Ok(())
}

pub fn save<C: DBConnection>(
    User {
        id,
        email,
        password,
        salt,
        subscription_expiration,
    }: User,
    tran: &C,
) -> Result<(), Box<dyn std::error::Error>> {
    tran.execute(
        "UPDATE users
                SET
                    email = ?1,
                    password = ?2,
                    salt = ?3,
                    subscription_expiration = ?4
                WHERE id = ?5;",
        params!(email, &password[..], &salt[..], subscription_expiration, id),
    )?;
    Ok(())
}

crate::basic_error!(DatabaseReadError, "Valid data not found in database");

pub fn get(email: &str, pool: &SqlPool) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let (salt, hash, id, subscription_expiration): (Vec<u8>, Vec<u8>, UserID, Option<i64>) =
        match pool.get()?.query_row(
            "SELECT salt, password, id, subscription_expiration
                    FROM users
                    WHERE email = ?1;",
            params![&email],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(Box::new(e)),
        };

    let salt = match salt.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return Err(Box::new(DatabaseReadError));
        }
    };

    let hash = match hash.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return Err(Box::new(DatabaseReadError));
        }
    };

    Ok(Some(User {
        id,
        email: email.to_owned(),
        password: hash,
        salt,
        subscription_expiration,
    }))
}

pub fn get_with_id(id: UserID, pool: &SqlPool) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let (salt, hash, email, subscription_expiration): (Vec<u8>, Vec<u8>, String, Option<i64>) =
        match pool.get()?.query_row(
            "SELECT salt, password, email, subscription_expiration
                    FROM users
                    WHERE id = ?1;",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(Box::new(e)),
        };

    let salt = match salt.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return Err(Box::new(DatabaseReadError));
        }
    };

    let hash = match hash.try_into() {
        Ok(arr) => arr,
        Err(_) => {
            return Err(Box::new(DatabaseReadError));
        }
    };

    Ok(Some(User {
        id,
        email: email,
        password: hash,
        salt,
        subscription_expiration,
    }))
}

crate::basic_error_with_dyn_message!(UserDoesNotExist, "User does not exist for id");

pub fn update_subscription_expiration(
    id: UserID,
    pool: &SqlPool,
    tran: &Transaction,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = crate::helper::get_millis_since_epoch()?;
    let mut current = match get_with_id(id, pool)? {
        Some(v) => v,
        None => return Err(Box::new(UserDoesNotExist::new(id.to_string()))),
    };
    // We collect all expiration dates, and set the lastest as the subscription_expiration field
    // Will later be expaned with more possible choices
    let data: [Option<i64>; 1] = [stripe_profile::get(id, pool)?
        .map(|s| {
            use stripe_profile::StripeSubscription;
            match s.subscription {
                Some(StripeSubscription { start, expiration })
                    if start < now && expiration > now =>
                {
                    Some(expiration)
                }
                _ => None,
            }
        })
        .flatten()];

    // Then we determine the last date of those found
    let last = data
        .iter()
        .fold(None, |last, current| match (last, *current) {
            (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            _ => None,
        });

    current.subscription_expiration = last;
    save(current, tran)?;
    Ok(())
}

#[allow(dead_code)]
pub async fn delete(user: User, pool: &SqlPool) -> Result<(), Box<dyn std::error::Error>> {
    let trans = SyncTransaction::new(pool)?;
    // First we remove all their tokens

    token::delete(user.id, &trans)?;

    // Then we try to remove their stripe profile
    let profile_removed = stripe_profile::delete_user(&user, &trans).await?;
    // Now if a profile existed, and was removesd, we must remeber this account is now gone from Stripe, so it's a serious issue if something goes wrong now

    match trans
        .execute("DELETE FROM users WHERE id = ?1", params![user.id])
        .and(Ok(trans.commit()))
    {
        Ok(_) => Ok(()),
        Err(e) if profile_removed => {
            let msg = format!(
                        "CRITICAL ERROR, PAYMENT DATA FOR USER WITH ID {} DELETED FROM STRIPE, BUT NOT DATABASE, Error was {}",
                        user.id,
                        e
                    );
            crate::basic_error_with_dyn_message!(ReallyBadError, "Really Really Bad Error");
            Err(Box::new(ReallyBadError::new(msg)))
        }
        Err(e) => Err(e),
    }
}
