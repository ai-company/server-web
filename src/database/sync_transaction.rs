use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Params};
use std::sync::{Arc, Mutex};

use super::{db_connection::DynResult, DBConnection, SqlPool};

pub struct SyncTransaction {
    con: Arc<Mutex<PooledConnection<SqliteConnectionManager>>>,
    done: bool, // prevent double rollback on drop, if transaction finalised
}

impl SyncTransaction {
    pub fn new(pool: &SqlPool) -> Result<SyncTransaction, Box<dyn std::error::Error>> {
        let con = pool.get()?;
        con.execute("BEGIN TRANSACTION", params!())?;
        Ok(SyncTransaction {
            con: Arc::new(Mutex::new(con)),
            done: false,
        })
    }

    pub fn commit(mut self) -> rusqlite::Result<usize> {
        self.done = true;
        self.con.lock().unwrap().execute("COMMIT", params![])
    }

    pub fn rollback(mut self) -> rusqlite::Result<usize> {
        self.done = true;
        self.con.lock().unwrap().execute("ROLLBACK", params![])
    }
}

impl DBConnection for SyncTransaction {
    fn execute<P: Params>(&self, sql: &str, params: P) -> DynResult<usize> {
        Ok(self.con.lock().unwrap().execute(sql, params)?)
    }

    fn query_row<T, P: Params, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.con.lock().unwrap().query_row(sql, params, f)?)
    }
}

impl Drop for SyncTransaction {
    fn drop(&mut self) {
        if !self.done {
            self.con
                .lock()
                .unwrap()
                .execute("ROLLBACK", params![])
                .unwrap();
        }
    }
}
