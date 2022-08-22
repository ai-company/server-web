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
pub type AdminID = i64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Admin {
    pub id: AdminID,
    pub username: String,
    pub password: String,
}

impl TryFrom<&Row<'_>> for Admin {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(row.column_index("id")?)?,
            username: row.get(row.column_index("username")?)?,
            password: row.get(row.column_index("password")?)?,
        })
    }
}

pub fn insert(
    username: &str,
    password: &str,
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let password = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

    pool.get()?.execute(
        "INSERT INTO admins
        (username, password)
        VALUES
        (:username, :password)",
        named_params! {
            ":username": username,
            ":password": password,
        },
    )?;

    println!("[ DB ] admin: inserted new");

    Ok(())
}

pub fn save<C: DBConnection>(data: &Admin, tran: &C) -> Result<(), Box<dyn std::error::Error>> {
    tran.execute(
        "UPDATE admins SET
            username = :username,
            password = :password
        WHERE
            id = :id",
        named_params! {
            ":id": data.id,
            ":username": data.username,
            ":password": data.password,
        },
    )?;

    println!("[ DB ] admin: save");

    Ok(())
}

crate::basic_error!(DatabaseReadError, "Valid data not found in database");

pub fn get(username: &str, pool: &SqlPool) -> Result<Option<Admin>, Box<dyn std::error::Error>> {
    let user = match pool.get()?.query_row(
        "SELECT *
        FROM admins
        WHERE username = ?",
        params![&username],
        |row| row.try_into(),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] admin: get by username");

    Ok(Some(user))
}

pub fn get_with_id(
    id: AdminID,
    pool: &SqlPool,
) -> Result<Option<Admin>, Box<dyn std::error::Error>> {
    let user = match pool.get()?.query_row(
        "SELECT *
        FROM admins
        WHERE id = ?",
        params![id],
        |row| row.try_into(),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] admin: get by id");

    Ok(Some(user))
}

crate::basic_error_with_dyn_message!(AdminDoesNotExist, "Admin does not exist for id");

/// Delete user
#[allow(dead_code)]
pub async fn delete(
    data: Admin,
    trans: &SyncTransaction,
) -> Result<(), Box<dyn std::error::Error>> {
    match trans.execute("DELETE FROM admins WHERE id = ?1", params![data.id]) {
        Ok(_) => {
            println!("[ DB ] deleted admin: {}", data.id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
