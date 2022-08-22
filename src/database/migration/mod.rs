//! Database migrations
//!
//! ```
//! use rusqlite::{params, Transaction};
//! use crate::database::initialize::MigrationResult;
//!
//! pub fn template(tran: &Transaction) -> MigrationResult {
//!     let queries = [
//!         ("//query", params![])
//!     ];
//!
//!     for (query, params) in queries.iter() {
//!         if let Err(e) = tran.execute(query, params) {
//!             return Err(e.to_string());
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod v001;
pub mod v002;
pub mod v003;
