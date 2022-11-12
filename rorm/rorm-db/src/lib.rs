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

pub use rorm_declaration::config::DatabaseDriver;
#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub use rorm_sql::{and, conditional, join_table, or, select, value};

#[cfg(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
))]
pub use crate::{
    database::{Database, DatabaseConfiguration},
    error::Error,
    row::Row,
};
