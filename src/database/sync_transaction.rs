use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::{Arc, Mutex};

use super::{db_connection::DynResult, DBConnection, SqlPool};

pub struct SyncTransaction {
    con: Arc<Mutex<PooledConnection<SqliteConnectionManager>>>,
}

impl SyncTransaction {
    pub fn new(pool: &SqlPool) -> Result<SyncTransaction, Box<dyn std::error::Error>> {
        let con = pool.get()?;
        con.execute("BEGIN TRANSACTION", params!())?;
        Ok(SyncTransaction {
            con: Arc::new(Mutex::new(con)),
        })
    }

    pub fn commit(self) -> rusqlite::Result<usize> {
        self.con
            .lock()
            .unwrap()
            .execute("TRANSACTION COMMIT", params![])
    }

    pub fn rollback(self) -> rusqlite::Result<usize> {
        self.con
            .lock()
            .unwrap()
            .execute("ROLLBACK TRANSACTION", params![])
    }
}

impl DBConnection for SyncTransaction {
    fn execute<P>(&self, sql: &str, params: P) -> DynResult<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
    {
        Ok(self.con.lock().unwrap().execute(sql, params)?)
    }

    fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.con.lock().unwrap().query_row(sql, params, f)?)
    }
}

impl Drop for SyncTransaction {
    fn drop(&mut self) {
        self.con
            .lock()
            .unwrap()
            .execute("ROLLBACK TRANSACTION", params![])
            .unwrap();
    }
}
