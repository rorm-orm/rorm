//! This crate is used as language independent base for building an orm.
//!
//! Rust specific features will be exposed through the `rorm` crate.
//! `rorm-lib` implements C bindings for this crate.
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![warn(missing_docs)]

#[cfg(all(
    not(cfg_auto_docs),
    any(
        all(
            feature = "tokio",
            feature = "async-std"
        ),
        all(
            feature = "native-tls",
            feature = "async-std-rustls"
        ),
    )
))]
compile_error!("Using multiple runtime / tls configurations at the same time is not allowed");

pub mod database;
pub mod error;

pub(crate) mod query_type;

pub mod choice;
pub mod executor;
pub mod row;
pub mod transaction;

#[cfg_attr(feature = "sqlx", path = "sqlx_impl/mod.rs")]
#[cfg_attr(not(feature = "sqlx"), path = "dummy_impl/mod.rs")]
pub(crate) mod internal;

/// Re-export [rorm-sql](rorm_sql)
pub mod sql {
    pub use rorm_sql::*;
}

pub use rorm_declaration::config::DatabaseDriver;

pub use crate::database::{Database, DatabaseConfiguration};
pub use crate::error::Error;
pub use crate::executor::Executor;
pub use crate::row::Row;
