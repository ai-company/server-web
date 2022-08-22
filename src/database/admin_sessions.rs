use rusqlite::params;

use crate::database::admin::AdminID;

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

pub type SessionToken = String;

pub fn insert<C: DBConnection>(
    con: C,
    token: &SessionToken,
    user_id: AdminID,
) -> Result<(), Box<dyn std::error::Error>> {
    con.execute(
        "INSERT INTO admin_sessions (token, user_id, expires_at) VALUES (?1, ?2, datetime('now', '1 day'))",
        params![&token[..], user_id],
    )?;

    println!("[ DB ] admin_sessions: inserted new {}", user_id);

    Ok(())
}

pub fn generate(
    pool: &SqlPool,
    user_id: AdminID,
) -> Result<SessionToken, Box<dyn std::error::Error>> {
    let token = nanoid::simple();

    // Will fail on token collision, though very unlikely
    // Token expires in 1 day after current UTC timestamp
    insert(pool.clone(), &token, user_id)?;

    Ok(token)
}

pub fn destroy(pool: &SqlPool, user_id: AdminID) -> Result<(), Box<dyn std::error::Error>> {
    let tran = SyncTransaction::new(pool)?;

    delete(user_id, &tran)?;

    tran.commit()?;

    println!("[ DB ] admin_sessions: destroyed {}", user_id);

    Ok(())
}

pub fn get_owner(
    pool: &SqlPool,
    token: SessionToken,
) -> Result<Option<AdminID>, Box<dyn std::error::Error>> {
    let admin = match pool.get()?.query_row(
        "SELECT user_id
        FROM admin_sessions
        WHERE token = ?1 AND expires_at > datetime('now')",
        params![&token[..]],
        |row| row.get(0),
    ) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] admin_sessions: get owner");

    admin
}

pub fn delete(owner: AdminID, trans: &SyncTransaction) -> Result<(), Box<dyn std::error::Error>> {
    trans.execute(
        "DELETE FROM admin_sessions WHERE user_id = ?1",
        params![owner],
    )?;

    println!("[ DB ] admin_sessions: delete {}", owner);

    Ok(())
}
