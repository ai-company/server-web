use super::{Connection, SqlPool};
use rusqlite::params;

const LATEST_SCHEME_VERSION: u32 = 1;

pub fn initialize(pool: SqlPool) -> Result<(), String> {
    if let Ok(con) = pool.get() {
        // First we get the current database scheme version
        if let Ok(version) = con.query_row("PRAGMA user_version;", params!(), |r| {
            r.get::<usize, u32>(0)
        }) {
            match version {
                0 => first_setup(con),
                // Add migrations here when needed
                LATEST_SCHEME_VERSION => Ok(()),
                _ => panic!(
                    "Unknown scheme version: {} \n latest known version is: {}",
                    version, LATEST_SCHEME_VERSION
                ),
            }
        } else {
            Err("Failed to get database scheme version".to_owned())
        }
    } else {
        Err("Failed to get database connection".to_owned())
    }
}

fn first_setup(mut con: Connection) -> Result<(), String> {
    let tran = match con.transaction() {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to create transaction with error: {}", e)),
    };

    type Params = &'static [&'static dyn rusqlite::ToSql];
    type Query = &'static str;

    const TABLE_CREATE_SCRIPTS: [Query; 3] = [
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

    const EMPTY_PARAMS: Params = &[];

    for script in TABLE_CREATE_SCRIPTS.iter() {
        if let Err(e) = tran.execute(script, EMPTY_PARAMS) {
            return Err(e.to_string());
        }
    }
    // We set the database version
    if let Err(e) = tran.pragma_update(None, "user_version", &LATEST_SCHEME_VERSION) {
        return Err(e.to_string());
    }

    match tran.commit() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
