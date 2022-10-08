//! This module defines a dummy with the same interface as our sqlx' AnyRow wrapper.
use crate::error::Error;

/// Represents a single row from the database.
pub struct Row(std::marker::PhantomData<()>);

impl Row {
    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name
    /// and a `usize` index can be used to access a column by position.
    pub fn get<T, I>(&self, _index: I) -> Result<T, Error>
    where
        I: ColumnIndex,
    {
        Err(Error::ConfigurationError(
            "Can't work with rows without sqlx".to_string(),
        ))
    }
}

/// Dummy trait to restrict [`Row::get`]'s index argument to strings and integers
pub trait ColumnIndex {}
impl ColumnIndex for usize {}
impl ColumnIndex for &str {}
