//! This module holds the high level model representation
//!
//! It adds:
//! - [`db_type`]: a type level version of [`imr::DbType`](crate::imr::DbType) to be used in generic type bound checks
//! - [`annotations`]: a type level version of [`imr::Annotation`](crate::imr::Annotation) to be used in generic type bound checks
//!
//! These features are split into different submodules to avoid name conflicts.

pub mod annotations;
pub mod db_type;

/// Trait for converting a hmr type into a imr one
pub trait AsImr {
    /// Imr type to convert to
    type Imr;

    /// Convert to imr type
    fn as_imr(&self) -> Self::Imr;
}

/// Location in the source code a model or field originates from
/// Used for better error messages in the migration tool
#[derive(Copy, Clone)]
pub struct Source {
    /// Filename of the source code of the model or field
    pub file: &'static str,
    /// Line of the model or field
    pub line: usize,
    /// Column of the model or field
    pub column: usize,
}

impl AsImr for Source {
    type Imr = crate::imr::Source;

    fn as_imr(&self) -> Self::Imr {
        crate::imr::Source {
            file: self.file.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}
