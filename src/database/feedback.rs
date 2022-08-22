use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feedback {
    pub id: i64,
    pub email: String,
    pub comment: String,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for Feedback {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(row.column_index("id")?)?,
            email: row.get(row.column_index("email")?)?,
            comment: row.get(row.column_index("comment")?)?,
            created_at: row.get(row.column_index("created_at")?)?,
        })
    }
}

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

    println!("[ DB ] feedback: insert");

    Ok(())
}

pub fn get_all(pool: &SqlPool) -> Result<Vec<Feedback>, Box<dyn std::error::Error>> {
    let connection = pool.get()?;
    let mut statement = connection.prepare("SELECT * FROM feedback")?;
    let rows = statement.query_map(params![], |row| row.try_into())?;

    let mut reports = Vec::new();
    for row in rows {
        reports.push(row?);
    }

    println!("[ DB ] feedback: get all");

    Ok(reports)
}
