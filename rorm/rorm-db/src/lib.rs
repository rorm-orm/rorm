//! This crate is used as language independent base for building an orm.
//!
//! Rust specific features will be exposed through the `rorm` crate.
//! `rorm-lib` implements C bindings for this crate.
#![warn(missing_docs)]

#[cfg(any(
    all(
        feature = "actix-rustls",
        any(
            feature = "actix-native-tls",
            feature = "tokio-native-tls",
            feature = "tokio-rustls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "actix-native-tls",
        any(
            feature = "tokio-native-tls",
            feature = "tokio-rustls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "tokio-rustls",
        any(
            feature = "tokio-native-tls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "tokio-native-tls",
        any(feature = "async-std-native-tls", feature = "async-std-rustls")
    ),
    all(feature = "async-std-native-tls", feature = "async-std-rustls")
))]
compile_error!("Using multiple runtime / tls configurations at the same time is not allowed");

#[cfg(not(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
)))]
compile_error!(
    r#"One of async-std-native-tls, async-std-rustls, tokio-native-tls, tokio-rustls, 
    actix-native-tls, actix-rustls is required"#
);

#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod database;
/**
Errors of rorm-db will be specified here.
 */
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod error;
/**
This module holds the results of a query
 */
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod result;
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod row;
/// This module holds the definition of transactions
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod transaction;
/// Utility functions
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub mod utils;

#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub use rorm_sql::{and, conditional, or, value};

#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub use crate::{database::Database, error::Error, row::Row};

/**
Representation of different backends
 */
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
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
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
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
