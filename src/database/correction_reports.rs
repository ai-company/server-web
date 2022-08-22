use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use super::{user::UserID, SyncTransaction};
use super::{DBConnection, SqlPool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorrectionReport {
    pub id: i64,
    pub email: String,
    pub report: String,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for CorrectionReport {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(row.column_index("id")?)?,
            email: row.get(row.column_index("email")?)?,
            report: row.get(row.column_index("report")?)?,
            created_at: row.get(row.column_index("created_at")?)?,
        })
    }
}

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

pub fn get_all(pool: &SqlPool) -> Result<Vec<CorrectionReport>, Box<dyn std::error::Error>> {
    let connection = pool.get()?;
    let mut statement = connection.prepare("SELECT * FROM correction_reports")?;
    let rows = statement.query_map(params![], |row| row.try_into())?;

    let mut reports = Vec::new();
    for row in rows {
        reports.push(row?);
    }

    println!("[ DB ] correction_reports: get all");

    Ok(reports)
}
