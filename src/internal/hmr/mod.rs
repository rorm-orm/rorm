//! This module holds the high level model representation
//!
//! It adds:
//! - [`annotations`]: a type level version of [`imr::Annotation`](crate::imr::Annotation) to be used in generic type bound checks
//!
//! These features are split into different submodules to avoid name conflicts.

use rorm_db::sql::value::NullType;

use crate::imr::DbType;

pub mod annotations;

/// Get the [`DbType`] a column with the given [`NullType`] is stored as
pub const fn db_type(null_type: NullType) -> DbType {
    match null_type {
        NullType::String => DbType::Text,
        NullType::Choice => DbType::Choices,
        NullType::I64 => DbType::Int64,
        NullType::I32 => DbType::Int32,
        NullType::I16 => DbType::Int16,
        NullType::Bool => DbType::Boolean,
        NullType::F64 => DbType::Double,
        NullType::F32 => DbType::Float,
        NullType::Binary => DbType::Binary,
        NullType::ChronoNaiveTime => DbType::Time,
        NullType::ChronoNaiveDate => DbType::Date,
        NullType::ChronoNaiveDateTime => DbType::DateTime,
        NullType::ChronoDateTime => DbType::DateTime,
        NullType::TimeDate => DbType::Date,
        NullType::TimeTime => DbType::Time,
        NullType::TimeOffsetDateTime => DbType::DateTime,
        NullType::TimePrimitiveDateTime => DbType::DateTime,
        NullType::Uuid => DbType::Uuid,
        NullType::UuidHyphenated => DbType::Uuid,
        NullType::UuidSimple => DbType::Uuid,
        NullType::JsonValue => DbType::Binary,
        #[cfg(feature = "postgres-only")]
        NullType::MacAddress => DbType::MacAddress,
        #[cfg(feature = "postgres-only")]
        NullType::IpNetwork => DbType::IpNetwork,
        #[cfg(feature = "postgres-only")]
        NullType::BitVec => DbType::BitVec,
    }
}

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
