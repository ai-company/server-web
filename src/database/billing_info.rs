use crate::{
    database::user::{User, UserID},
    route::user::signup::UserCreationRequest,
};

use super::{DBConnection, SqlPool, SyncTransaction};
use rusqlite::{named_params, params, Row};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingInfo {
    pub id: i64,
    pub user_id: i64,
    pub fullname: String,
    pub address: String,
    pub city: String,
    pub zip: String,
    pub country: String,
    pub cvr: Option<String>,
    pub studentid: Option<Vec<u8>>,
}

impl TryFrom<&Row<'_>> for BillingInfo {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(BillingInfo {
            id: row.get(row.column_index("id")?)?,
            user_id: row.get(row.column_index("user_id")?)?,
            fullname: row.get(row.column_index("fullname")?)?,
            address: row.get(row.column_index("address")?)?,
            city: row.get(row.column_index("city")?)?,
            zip: row.get(row.column_index("zip")?)?,
            country: row.get(row.column_index("country")?)?,
            cvr: row.column_index("cvr").and_then(|i| row.get(i)).ok(),
            studentid: row.column_index("studentid").and_then(|i| row.get(i)).ok(),
        })
    }
}

pub fn insert(
    data: &UserCreationRequest,
    user: &User,
    pool: &SqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    pool.get()?.execute(
        "INSERT INTO billing_info
            (user_id, fullname, cvr, studentid, address, city, zip, country)
        VALUES
            (:user_id, :fullname, :cvr, :studentid, :address, :city, :zip, :country)",
        named_params! {
            ":user_id": user.id,
            ":fullname": data.fullname,
            ":cvr": data.cvr,
            ":studentid": data.studentid,
            ":address": data.address,
            ":city": data.city,
            ":zip": data.zip,
            ":country": data.country,
        },
    )?;

    Ok(())
}

pub fn save<C: DBConnection>(
    data: &BillingInfo,
    tran: &C,
) -> Result<(), Box<dyn std::error::Error>> {
    tran.execute(
        "UPDATE billing_info SET
            fullname = :fullname,
            cvr = :cvr,
            studentid = :studentid,
            address = :address,
            city = :city,
            zip = :zip,
            country = :country
        WHERE id = :id;",
        named_params! {
            ":fullname": data.fullname,
            ":cvr": data.cvr,
            ":studentid": data.studentid,
            ":address": data.address,
            ":city": data.city,
            ":zip": data.zip,
            ":country": data.country,
            ":id": data.id,
        },
    )?;

    Ok(())
}

crate::basic_error!(DatabaseReadError, "Valid data not found in database");

pub fn get_with_id(
    id: UserID,
    pool: &SqlPool,
) -> Result<Option<BillingInfo>, Box<dyn std::error::Error>> {
    let user = match pool.get()?.query_row(
        "SELECT *
        FROM billing_info
        WHERE user_id = ?",
        params![id],
        |row| row.try_into(),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    Ok(Some(user))
}

crate::basic_error_with_dyn_message!(UserDoesNotExist, "User does not exist for id");

#[allow(dead_code)]
pub async fn delete(
    user_id: UserID,
    trans: &SyncTransaction,
) -> Result<(), Box<dyn std::error::Error>> {
    match trans.execute(
        "DELETE FROM billing_info WHERE user_id = ?1",
        params![user_id],
    ) {
        Ok(_) => {
            println!("[ DB ] deleted user billing info: {}", user_id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
