use rusqlite::params;

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

pub fn insert<C: DBConnection>(
    con: C,
    email: &str,
    comment: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[ DB ] Insert feedback");
    con.execute(
        "INSERT INTO feedback (email, comment) VALUES (?1, ?2)",
        params![email, comment],
    )?;
    Ok(())
}
