//! This crate holds all declarative parts of rorm which do not depend
//! on another crate.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

pub mod config;
/// This module holds the internal model representation
pub mod imr;
pub mod lints;
/// This module holds the definition of migration files
pub mod migration;
