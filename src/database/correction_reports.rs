use rusqlite::params;

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

pub fn insert<C: DBConnection>(
    con: C,
    email: &str,
    report: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[ DB ] Insert correction report");
    con.execute(
        "INSERT INTO correction_reports (email, report) VALUES (?1, ?2)",
        params![email, report],
    )?;

    println!("[ DB ] correction_reports: insert");

    Ok(())
}
