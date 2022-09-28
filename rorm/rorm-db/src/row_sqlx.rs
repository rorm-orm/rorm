//! This module defines a wrapper for sqlx's AnyRow
use crate::error::Error;
use sqlx::any::AnyRow;
use sqlx::Any as AnyDb;
use sqlx::{ColumnIndex, Decode, Type};

/// Represents a single row from the database.
pub struct Row(AnyRow);

impl Row {
    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name
    /// and a `usize` index can be used to access a column by position.
    pub fn get<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        T: Decode<'r, AnyDb> + Type<AnyDb>,
        I: ColumnIndex<AnyRow>,
    {
        <AnyRow as sqlx::Row>::try_get(&self.0, index).map_err(Error::SqlxError)
    }
}

impl From<AnyRow> for Row {
    fn from(any_row: AnyRow) -> Self {
        Row(any_row)
    }
}
