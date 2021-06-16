use r2d2_sqlite::SqliteConnectionManager;

pub type SqlPool = r2d2::Pool<SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

mod initialize;
pub use initialize::initialize;

mod sync_transaction;
pub use sync_transaction::SyncTransaction;

mod db_connection;
pub use db_connection::DBConnection;

pub mod stripe_profile;
pub mod token;
pub mod user;
