use std::any::type_name;

use sqlx::{ColumnIndex, Type, TypeInfo, ValueRef};

use crate::row::{RowError, RowIndex};

pub(crate) fn try_get<'r, 'i, R, T>(row: &'r R, index: RowIndex<'i>) -> Result<T, RowError<'i>>
where
    R: sqlx::Row,
    usize: ColumnIndex<R>,
    &'i str: ColumnIndex<R>,
    T: sqlx::Decode<'r, R::Database> + Type<R::Database>,
{
    let value = match index {
        RowIndex::Position(index) => row.try_get_raw(index),
        RowIndex::Name(index) => row.try_get_raw(index),
    }
    .map_err(|error| match error {
        sqlx::Error::ColumnIndexOutOfBounds { .. } | sqlx::Error::ColumnNotFound(_) => {
            RowError::NotFound { index }
        }
        _ => RowError::Unknown {
            index,
            source: Box::new(error),
        },
    })?;

    if !value.is_null() {
        let ty = value.type_info();

        if !ty.is_null() && !T::compatible(&ty) {
            return Err(RowError::MismatchedTypes {
                index,
                rust_type: type_name::<T>(),
            });
        }
    }

    T::decode(value).map_err(|source| {
        if source.is::<sqlx::error::UnexpectedNullError>() {
            RowError::UnexpectedNull { index }
        } else {
            RowError::Decode { index, source }
        }
    })
}
