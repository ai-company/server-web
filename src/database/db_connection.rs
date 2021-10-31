use std::sync::Arc;

use rusqlite::{Connection, Params, Transaction};

use super::SqlPool;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub trait DBConnection {
    fn execute<P: Params>(&self, sql: &str, params: P) -> DynResult<usize>;

    fn query_row<T, P: Params, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>;
}

impl DBConnection for Transaction<'_> {
    fn execute<P: Params>(&self, sql: &str, params: P) -> DynResult<usize> {
        Ok(Connection::execute(&self, sql, params)?)
    }

    fn query_row<T, P: Params, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(Connection::query_row(&self, sql, params, f)?)
    }
}

impl DBConnection for SqlPool {
    fn execute<P: Params>(&self, sql: &str, params: P) -> DynResult<usize> {
        Ok(self.get()?.execute(sql, params)?)
    }

    fn query_row<T, P: Params, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.get()?.query_row(sql, params, f)?)
    }
}

impl DBConnection for Arc<SqlPool> {
    fn execute<P: Params>(&self, sql: &str, params: P) -> DynResult<usize> {
        Ok(self.get()?.execute(sql, params)?)
    }

    fn query_row<T, P: Params, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.get()?.query_row(sql, params, f)?)
    }
}
