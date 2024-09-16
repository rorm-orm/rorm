//! This module defines a wrapper for sqlx's AnyRow

pub use self::error::*;
pub use self::index::*;
use crate::internal;

mod error;
mod index;

/// Represents a single row from the database.
pub struct Row(pub(crate) internal::row::Impl);

impl Row {
    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name
    /// and a `usize` index can be used to access a column by position.
    pub fn get<'r, 'i, T>(&'r self, index: impl Into<RowIndex<'i>>) -> Result<T, RowError<'i>>
    where
        T: Decode<'r>,
    {
        internal::row::get(self, index.into())
    }
}

/// Something which can be decoded from a [`Row`]'s cell.
#[cfg(feature = "sqlx")]
pub trait Decode<'r>: internal::any::AnyType + internal::any::AnyDecode<'r> {}
/// Something which can be decoded from a [`Row`]'s cell.
#[cfg(not(feature = "sqlx"))]
pub trait Decode<'r> {}

/// Something which can be decoded from a [`Row`]'s cell without borrowing.
pub trait DecodeOwned: for<'r> Decode<'r> {}
impl<T: for<'r> Decode<'r>> DecodeOwned for T {}

#[cfg(feature = "sqlx")]
impl<'r, T: internal::any::AnyType + internal::any::AnyDecode<'r>> Decode<'r> for T {}
#[cfg(not(feature = "sqlx"))]
impl<'r, T> Decode<'r> for T {}
