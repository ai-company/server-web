use super::SqlPool;
use rusqlite::{params, Transaction};

use super::migration;

pub type MigrationResult = Result<(), String>;
pub type Migration = fn(&Transaction) -> MigrationResult;

const SCHEME_MIGRATIONS: [Migration; 3] = [
    //
    migration::v001::first_setup,
    migration::v002::add_feedback,
    migration::v003::add_admins,
];

const SCHEME_VERSION: usize = SCHEME_MIGRATIONS.len();

pub fn initialize(pool: SqlPool) -> MigrationResult {
    println!("[ DB ] Initialize");

    if let Ok(mut con) = pool.get() {
        // Get the current database scheme version
        // `PRAGMA user_version` is a SQLite specific variable for application use
        if let Ok(version) = con.query_row("PRAGMA user_version;", params!(), |r| {
            r.get::<usize, u32>(0)
        }) {
            let version = version as usize;

            // Make sure we don't have a db from future
            if version > SCHEME_VERSION {
                panic!(
                    "Unknown scheme version: {} \n latest known version is: {}",
                    version, SCHEME_VERSION
                )
            }

            // Execute migrations, if any
            for migrate_to in version..SCHEME_VERSION {
                // Create transaction
                let tran = match con.transaction() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(format!("Failed to create transaction with error: {}", e))
                    }
                };

                // Execute migration
                if let Err(e) = SCHEME_MIGRATIONS[migrate_to](&tran) {
                    println!(
                        "[ DB ] Failed migration v{} -> v{}",
                        migrate_to,
                        migrate_to + 1
                    );

                    return Err(e);
                };

                // Set the database version
                if let Err(e) = tran.pragma_update(None, "user_version", &(migrate_to as u32 + 1)) {
                    return Err(e.to_string());
                }

                // Commit transaction
                if let Err(e) = tran.commit() {
                    return Err(e.to_string());
                }

                println!("[ DB ] Migrate v{} -> v{}", migrate_to, migrate_to + 1);
            }

            Ok(())
        } else {
            Err("Failed to get database scheme version".to_owned())
        }
    } else {
        Err("Failed to get database connection".to_owned())
    }
}
