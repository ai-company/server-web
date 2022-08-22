use rusqlite::{params, Transaction};

use crate::database::initialize::MigrationResult;

pub fn add_admins(tran: &Transaction) -> MigrationResult {
    let queries = [
        (
            "CREATE TABLE admins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                created_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        ),
        (
            "CREATE TABLE admin_sessions (
                token TEXT UNIQUE PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at DATE NOT NULL,
                FOREIGN KEY(user_id) REFERENCES admins(id) ON DELETE CASCADE
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
