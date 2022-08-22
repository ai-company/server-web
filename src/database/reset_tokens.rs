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
        "INSERT INTO reset_tokens (token, user_id, expires_at) VALUES (?1, ?2, datetime('now', '1 hour'))",
        params![&token[..], user_id],
    )?;

    println!("[ DB ] reset_tokens: insert");

    Ok(())
}

pub fn generate(pool: &SqlPool, user_id: UserID) -> Result<Token, Box<dyn std::error::Error>> {
    let token = nanoid::generate(48);

    // Will fail on token collision, though very unlikely
    // Token expires in 1 hour after current UTC timestamp
    insert(pool.clone(), &token, user_id)?;

    Ok(token)
}

pub fn destroy(pool: &SqlPool, user_id: UserID) -> Result<(), Box<dyn std::error::Error>> {
    let tran = SyncTransaction::new(pool)?;

    delete(user_id, &tran)?;

    tran.commit()?;

    println!("[ DB ] reset_tokens: destroy");

    Ok(())
}

pub fn get_owner(
    pool: &SqlPool,
    token: Token,
) -> Result<Option<UserID>, Box<dyn std::error::Error>> {
    let owner = match pool.get()?.query_row(
        "SELECT user_id
        FROM reset_tokens
        WHERE token = ?1 AND expires_at > datetime('now')",
        params![&token[..]],
        |row| row.get(0),
    ) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] reset_tokens: get owner");

    owner
}

pub fn delete(owner: UserID, trans: &SyncTransaction) -> Result<(), Box<dyn std::error::Error>> {
    trans.execute(
        "DELETE FROM reset_tokens WHERE user_id = ?1",
        params![owner],
    )?;

    println!("[ DB ] reset_tokens: delete");

    Ok(())
}
