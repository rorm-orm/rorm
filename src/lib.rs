#![cfg_attr(docsrs, feature(doc_cfg))]

/// Reexport the config
pub use rorm_declaration::config;

/// Reexports for executing the defined cli parser via another
pub mod entry;
/// This module is used for creating a configuration file that can be used by the
/// binary version.
pub mod init;
/// This module handles the creation of migration files
pub mod make_migrations;
/// This module is used for applying migrations
pub mod migrate;

mod linter;
mod squash_migrations;
mod utils;
