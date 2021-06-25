use rusqlite::params;

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

pub type Token = String;

pub fn insert<C: DBConnection>(
    con: C,
    token: &Token,
    user_id: UserID,
) -> Result<(), Box<dyn std::error::Error>> {
    con.execute(
        "INSERT INTO tokens
                (token, user, expiration)
                VALUES (?1, ?2, datetime('now', '30 day'));",
        params![&token[..], user_id],
    )?;
    Ok(())
}

pub fn generate(pool: &SqlPool, user_id: UserID) -> Result<Token, Box<dyn std::error::Error>> {
    let token = nanoid::simple();

    // Will fail on token collision, though very unlikely
    // Token expires in 30 days after current UTC timestamp
    insert(pool.clone(), &token, user_id)?;

    Ok(token)
}

pub fn destroy(pool: &SqlPool, user_id: UserID) -> Result<(), Box<dyn std::error::Error>> {
    let tran = SyncTransaction::new(pool)?;

    delete(user_id, &tran)?;

    tran.commit()?;

    Ok(())
}

pub fn get_owner(
    pool: &SqlPool,
    token: Token,
) -> Result<Option<UserID>, Box<dyn std::error::Error>> {
    match pool.get()?.query_row(
        "SELECT user
                FROM tokens
                WHERE token = ?1
                    AND expiration > datetime('now')",
        params![&token[..]],
        |row| Ok(row.get(0)?),
    ) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn delete(owner: UserID, trans: &SyncTransaction) -> Result<(), Box<dyn std::error::Error>> {
    trans.execute("DELETE FROM tokens WHERE user = ?1", params![owner])?;
    Ok(())
}
