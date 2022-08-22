use crate::route::user::signup::UserCreationRequest;

use super::{stripe_profile, DBConnection, SqlPool, SyncTransaction};
use rusqlite::{named_params, params, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    convert::{TryFrom, TryInto},
    fs::File,
    io::{self, BufRead},
    path::Path,
};

// Never negative but row.get in rusqlite does not work for u64 for some reason
pub type UserID = i64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserID,
    pub email: String,
    pub password: String,
    pub tier: Option<String>,
    pub verified: bool,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for User {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(row.column_index("id")?)?,
            email: row.get(row.column_index("email")?)?,
            password: row.get(row.column_index("password")?)?,
            tier: row.column_index("tier").and_then(|i| row.get(i)).ok(),
            verified: row.get(row.column_index("verified")?)?,
            created_at: row.get(row.column_index("created_at")?)?,
        })
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn is_invited(email: &str) -> bool {
    let email_hash = hex::encode(Sha256::digest(email.as_bytes()));

    if let Ok(lines) = read_lines("db/emails.txt") {
        for invited_email in lines.flatten() {
            if invited_email.trim() == email_hash {
                return true;
            }
        }
    }

    return false;
}

pub fn insert(
    data: &UserCreationRequest,
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    pool.get()?.execute(
        "INSERT INTO users
        (email, password, tier)
        VALUES
        (:email, :password, :tier)",
        named_params! {
            ":email": data.email,
            ":password": data.password,
            ":tier": data.tier,
        },
    )?;

    println!("[ DB ] user: inserted new");

    Ok(())
}

pub fn verify(data: &User, pool: &SqlPool) -> Result<(), Box<dyn std::error::Error>> {
    pool.get()?.execute(
        "UPDATE users SET
            verified = TRUE
        WHERE
            id = :id",
        named_params! {
            ":id": data.id,
        },
    )?;

    println!("[ DB ] user: verify");

    Ok(())
}

pub fn save<C: DBConnection>(data: &User, tran: &C) -> Result<(), Box<dyn std::error::Error>> {
    tran.execute(
        "UPDATE users SET
            email = :email,
            tier = :tier,
            password = :password
            verified = :verified
        WHERE
            id = :id",
        named_params! {
            ":id": data.id,
            ":email": data.email,
            ":tier": data.tier,
            ":password": data.password,
            ":verified": data.verified,
        },
    )?;

    println!("[ DB ] user: save");

    Ok(())
}

crate::basic_error!(DatabaseReadError, "Valid data not found in database");

pub fn get(email: &str, pool: &SqlPool) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let user = match pool.get()?.query_row(
        "SELECT *
        FROM users
        WHERE email = ?",
        params![&email],
        |row| row.try_into(),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] user: get by email");

    Ok(Some(user))
}

pub fn get_all(pool: &SqlPool) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let connection = pool.get()?;
    let mut statement = connection.prepare("SELECT * FROM users")?;
    let rows = statement.query_map(params![], |row| row.try_into())?;

    let mut users = Vec::new();
    for row in rows {
        users.push(row?);
    }

    println!("[ DB ] user: get all");

    Ok(users)
}

pub fn get_with_id(id: UserID, pool: &SqlPool) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let user = match pool.get()?.query_row(
        "SELECT *
        FROM users
        WHERE id = ?",
        params![id],
        |row| row.try_into(),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] user: get by id");

    Ok(Some(user))
}

crate::basic_error_with_dyn_message!(UserDoesNotExist, "User does not exist for id");

/// Delete user
///
/// Actions performed:
/// - Stripe remote profile deleted
/// - Stripe local info deleted
/// - User profile deleted
/// - User session tokens deleted (db trigger)
/// - User billing info deleted (db trigger)
#[allow(dead_code)]
pub async fn delete(data: User, trans: &SyncTransaction) -> Result<(), Box<dyn std::error::Error>> {
    let profile_removed = stripe_profile::delete_user(&data, trans).await?;

    // If a profile existed, and was removed, we must remember this account is now gone from Stripe, so it's a serious issue if something goes wrong now

    println!("[ DB ] deleted stripe profile for user: {}", data.id);

    match trans.execute("DELETE FROM users WHERE id = ?1", params![data.id]) {
        Ok(_) => {
            println!("[ DB ] deleted user: {}", data.id);
            Ok(())
        }
        Err(e) if profile_removed => {
            let msg = format!(
                "CRITICAL ERROR, PAYMENT DATA FOR USER WITH ID {} DELETED FROM STRIPE, BUT NOT DATABASE, Error was {}",
                data.id,
                e
            );
            crate::basic_error_with_dyn_message!(ReallyBadError, "Really Really Bad Error");
            Err(Box::new(ReallyBadError::new(msg)))
        }
        Err(e) => Err(e),
    }
}
