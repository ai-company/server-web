use rusqlite::{params, Transaction};

use crate::database::initialize::MigrationResult;

pub fn add_feedback(tran: &Transaction) -> MigrationResult {
    let queries = [
        (
            "CREATE TABLE feedback (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL,
                comment TEXT NOT NULL,
                created_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        ),
        (
            "CREATE TABLE correction_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL,
                report TEXT NOT NULL,
                created_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        ),
    ];

    for (query, params) in queries.iter() {
        if let Err(e) = tran.execute(query, *params) {
            return Err(e.to_string());
        }
    }

    Ok(())
}
