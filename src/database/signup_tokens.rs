use rusqlite::{named_params, params};

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

pub type Token = String;
pub type SignupKey = String;

pub fn insert<C: DBConnection>(
    con: C,
    token: &Token,
    session_key: &SignupKey,
    user_id: UserID,
) -> Result<(), Box<dyn std::error::Error>> {
    con.execute(
        "INSERT INTO signup_tokens
            (token, session_key, user_id, expires_at)
        VALUES
            (:token, :session_key, :user_id, datetime('now', '1 hour'))",
        named_params! {
            ":token": token,
            ":session_key": session_key,
            ":user_id": user_id
        },
    )?;

    println!("[ DB ] signup_token: insert {}", user_id);

    Ok(())
}

pub fn generate_signup_key() -> SignupKey {
    nanoid::generate(16)
}

pub fn generate(
    pool: &SqlPool,
    user_id: UserID,
) -> Result<(Token, SignupKey), Box<dyn std::error::Error>> {
    let token = nanoid::generate(48);
    let session_key = generate_signup_key();

    // Will fail on token collision, though very unlikely
    // Token expires in 1 hour after current UTC timestamp
    insert(pool.clone(), &token, &session_key, user_id)?;

    Ok((token, session_key))
}

pub fn destroy(pool: &SqlPool, user_id: UserID) -> Result<(), Box<dyn std::error::Error>> {
    let tran = SyncTransaction::new(pool)?;

    delete(user_id, &tran)?;

    tran.commit()?;

    println!("[ DB ] signup_token: destroy {}", user_id);

    Ok(())
}

pub fn get_by_signup_key(
    pool: &SqlPool,
    signup_key: &SignupKey,
) -> Result<Option<Token>, Box<dyn std::error::Error>> {
    let token = match pool.get()?.query_row(
        "SELECT token
        FROM signup_tokens
        WHERE session_key = ?1 AND expires_at > datetime('now')",
        params![&signup_key[..]],
        |row| row.get(0),
    ) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] signup_token: get by signup key");

    token
}

pub fn get_owner(
    pool: &SqlPool,
    token: &Token,
) -> Result<Option<UserID>, Box<dyn std::error::Error>> {
    let owner = match pool.get()?.query_row(
        "SELECT user_id
        FROM signup_tokens
        WHERE token = ?1 AND expires_at > datetime('now')",
        params![&token[..]],
        |row| row.get(0),
    ) {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    };

    println!("[ DB ] signup_token: get owner");

    owner
}

pub fn delete(owner: UserID, trans: &SyncTransaction) -> Result<(), Box<dyn std::error::Error>> {
    trans.execute(
        "DELETE FROM signup_tokens WHERE user_id = ?1",
        params![owner],
    )?;

    println!("[ DB ] signup_token: delete");

    Ok(())
}
