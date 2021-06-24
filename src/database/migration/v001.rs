use rusqlite::{params, Transaction};

use crate::database::initialize::MigrationResult;

pub fn first_setup(tran: &Transaction) -> MigrationResult {
    let queries = [
        "create table users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            subscription_expiration DATE
        );",
        "create table tokens (
            token TEXT UNIQUE PRIMARY KEY,
            user INTEGER NOT NULL,
            expiration DATE NOT NULL,
            FOREIGN KEY(user) REFERENCES users(id)
        );",
        "create table stripe_profiles (
            stripe_id TEXT UNIQUE PRIMARY KEY,
            user INTEGER UNIQUE NOT NULL,
            stripe_subscription_expiration DATE,
            stripe_subscription_start DATE,
            last_updated DATE NOT NULL,
            FOREIGN KEY(user) REFERENCES users(id)
        );",
    ];

    let params = params![];

    for script in queries.iter() {
        if let Err(e) = tran.execute(script, params) {
            return Err(e.to_string());
        }
    }

    Ok(())
}
