use rusqlite::{params, Transaction};

use crate::database::initialize::MigrationResult;

pub fn template(tran: &Transaction) -> MigrationResult {
    let queries = [];
    let params = params![];

    for query in queries.iter() {
        if let Err(e) = tran.execute(query, params) {
            return Err(e.to_string());
        }
    }

    Ok(())
}
