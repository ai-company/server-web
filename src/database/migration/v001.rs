use rusqlite::{params, Transaction};

use crate::database::initialize::MigrationResult;

pub fn first_setup(tran: &Transaction) -> MigrationResult {
    let queries = [
        (
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                tier TEXT,
                verified BOOLEAN DEFAULT FALSE,
                created_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        ),
        (
            "CREATE TABLE signup_tokens (
                token TEXT UNIQUE PRIMARY KEY,
                session_key TEXT UNIQUE,
                user_id INTEGER NOT NULL,
                expires_at DATE NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            params![],
        ),
        (
            "CREATE TABLE reset_tokens (
                token TEXT UNIQUE PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at DATE NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            params![],
        ),
        (
            "CREATE TABLE billing_info (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                fullname TEXT NOT NULL,
                address TEXT NOT NULL,
                city TEXT NOT NULL,
                zip TEXT NOT NULL,
                country TEXT NOT NULL,
                cvr TEXT,
                studentid BLOB,
                updated_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            params![],
        ),
        (
            "CREATE TRIGGER on_update_billing_info AFTER UPDATE OF
                user_id,
                fullname,
                address,
                city,
                zip,
                country,
                cvr,
                studentid
            ON billing_info
            FOR EACH ROW BEGIN
                UPDATE billing_info
                SET updated_at = DATETIME('now')
                WHERE rowid = new.rowid;
            END",
            params![],
        ),
        (
            "CREATE TABLE sessions (
                token TEXT UNIQUE PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at DATE NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            params![],
        ),
        (
            "CREATE TABLE stripe_profiles (
                id TEXT UNIQUE PRIMARY KEY,
                user_id INTEGER UNIQUE NOT NULL,
                subscription_start INTEGER,
                subscription_end INTEGER,
                updated_at DATE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            params![],
        ),
        (
            "CREATE TRIGGER on_update_stripe_profiles AFTER UPDATE OF
                id,
                user_id,
                subscription_start,
                subscription_end
            ON stripe_profiles
            FOR EACH ROW BEGIN
                UPDATE billing_info
                SET updated_at = DATETIME('now')
                WHERE rowid = new.rowid;
            END",
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
