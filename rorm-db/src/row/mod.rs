//! This module defines a wrapper for sqlx's AnyRow

pub use self::error::*;
pub use self::index::*;
use crate::internal::any::{AnyDecode, AnyRow, AnyType};
use crate::row::get::try_get;

mod error;
mod get;
mod index;

/// Represents a single row from the database.
pub struct Row(pub(crate) AnyRow);

impl Row {
    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name
    /// and a `usize` index can be used to access a column by position.
    pub fn get<'r, 'i, T>(&'r self, index: impl Into<RowIndex<'i>>) -> Result<T, RowError<'i>>
    where
        T: Decode<'r>,
    {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyRow::Postgres(row) => try_get(row, index.into()),
            #[cfg(feature = "sqlite")]
            AnyRow::Sqlite(row) => try_get(row, index.into()),
        }
    }
}

/// Something which can be decoded from a [`Row`]'s cell.
pub trait Decode<'r>: AnyType + AnyDecode<'r> {}
impl<'r, T: AnyType + AnyDecode<'r>> Decode<'r> for T {}

/// Something which can be decoded from a [`Row`]'s cell without borrowing.
pub trait DecodeOwned: for<'r> Decode<'r> {}
impl<T: for<'r> Decode<'r>> DecodeOwned for T {}
