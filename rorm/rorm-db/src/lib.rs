//! This crate is used as language independent base for building an orm.
//!
//! Rust specific features will be exposed through the `rorm` crate.
//! `rorm-lib` implements C bindings for this crate.
#![warn(missing_docs)]

/**
Errors of rorm-db will be specified here.
*/
pub mod error;

#[cfg(feature = "sqlx-dep")]
/**
This module holds the definitions of queries and their results
*/
pub mod query;

#[cfg(feature = "sqlx-dep")]
/**
This module holds the results of a query
*/
pub mod result;

#[cfg(feature = "sqlx-dep")]
#[path = "row_sqlx.rs"]
pub mod row;
#[cfg(not(feature = "sqlx-dep"))]
#[path = "row_dummy.rs"]
pub mod row;

#[cfg(feature = "sqlx-dep")]
/// Utility functions
pub mod utils;

pub use rorm_sql::conditional;
pub use rorm_sql::value;
pub use rorm_sql::{and, or};

pub use crate::error::Error;
pub use crate::row::Row;

/**
Representation of different backends
*/
pub enum DatabaseBackend {
    /// SQLite database backend
    SQLite,
    /// Postgres database backend
    Postgres,
    /// MySQL / MariaDB database backend
    MySQL,
}

/**
Configuration to create a database connection.

If [DatabaseBackend::SQLite] is used as backend, `name` specifies the filename.
`host`, `port`, `user`, `password` is not used in this case.

If [DatabaseBackend::Postgres] or [DatabaseBackend::MySQL] is used, `name` specifies the
database to connect to.

`min_connections` and `max_connections` must be greater than 0
and `max_connections` must be greater or equals `min_connections`.
*/
pub struct DatabaseConfiguration {
    /// Specifies the driver that will be used
    pub backend: DatabaseBackend,
    /// Name of the database, in case of [DatabaseBackend::SQLite] name of the file.
    pub name: String,
    /// Host to connect to. Not used in case of [DatabaseBackend::SQLite].
    pub host: String,
    /// Port to connect to. Not used in case of [DatabaseBackend::SQLite].
    pub port: u16,
    /// Username to authenticate with. Not used in case of [DatabaseBackend::SQLite].
    pub user: String,
    /// Password to authenticate with. Not used in case of [DatabaseBackend::SQLite].
    pub password: String,
    /// Minimal connections to initialize upfront.
    pub min_connections: u32,
    /// Maximum connections that allowed to be created.
    pub max_connections: u32,
}

#[cfg(feature = "sqlx-dep")]
#[path = "database_sqlx.rs"]
pub mod database;
#[cfg(not(feature = "sqlx-dep"))]
#[path = "database_dummy.rs"]
pub mod database;
pub use database::Database;
