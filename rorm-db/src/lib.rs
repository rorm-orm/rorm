//! This crate is used as language independent base for building an orm.
//!
//! Rust specific features will be exposed through the `rorm` crate.
//! `rorm-lib` implements C bindings for this crate.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(all(not(feature = "postgres"), not(feature = "sqlite")))]
compile_error!("Can't compile with sqlx without any database");

pub mod database;
pub mod error;

pub(crate) mod query_type;

pub mod choice;
pub mod executor;
pub(crate) mod futures_util;
pub mod row;
pub mod transaction;

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
