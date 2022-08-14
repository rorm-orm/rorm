//! This crate holds all declarative parts of rorm which do not depend
//! on another crate.
#![warn(missing_docs)]

/// This module holds the internal model representation
pub mod imr;

/// This module holds the definition of migration files
pub mod migration;

/// This module holds traits and structs for working with models
pub mod model;
