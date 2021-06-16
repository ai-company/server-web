use rusqlite::{Connection, Transaction};

use super::SqlPool;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub trait DBConnection {
    fn execute<P>(&self, sql: &str, params: P) -> DynResult<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql;

    fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>;
}

impl DBConnection for Transaction<'_> {
    fn execute<P>(&self, sql: &str, params: P) -> DynResult<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
    {
        Ok(Connection::execute(&self, sql, params)?)
    }

    fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(Connection::query_row(&self, sql, params, f)?)
    }
}

impl DBConnection for SqlPool {
    fn execute<P>(&self, sql: &str, params: P) -> DynResult<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
    {
        Ok(self.get()?.execute(sql, params)?)
    }

    fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> DynResult<T>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.get()?.query_row(sql, params, f)?)
    }
}
