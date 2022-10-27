//! This module defines a wrapper for sqlx's AnyRow
use sqlx::any::AnyRow;
use sqlx::Any as AnyDb;
use sqlx::{ColumnIndex, Decode, Type};

use crate::error::Error;

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

/// Something which can be decoded from a [row](Row).
///
/// Auto-implemented for tuples of size 8 or less.
pub trait FromRow: Sized {
    /// Try decoding a [row](Row) into `Self`.
    fn from_row(row: Row) -> Result<Self, Error>;
}

macro_rules! impl_from_row {
    ($($generic:ident@$index:literal),+) => {
        impl<$($generic),+> FromRow for ($($generic,)+)
        where
            $(
                $generic: Type<AnyDb> + for<'r> Decode<'r, AnyDb>,
            )+
        {
            fn from_row(row: Row) -> Result<Self, Error> {
                Ok((
                    $(
                        row.get::<$generic, usize>($index)?,
                    )+
                ))
            }
        }
    };
}
impl_from_row!(A@0);
impl_from_row!(A@0, B@1);
impl_from_row!(A@0, B@1, C@2);
impl_from_row!(A@0, B@1, C@2, D@3);
impl_from_row!(A@0, B@1, C@2, D@3, E@4);
impl_from_row!(A@0, B@1, C@2, D@3, E@4, F@5);
impl_from_row!(A@0, B@1, C@2, D@3, E@4, F@5, G@6);
impl_from_row!(A@0, B@1, C@2, D@3, E@4, F@5, G@6, H@7);
