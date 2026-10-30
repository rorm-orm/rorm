use std::any::type_name;

use sqlx::{ColumnIndex, Type, TypeInfo, ValueRef};

use crate::row::{RowError, RowIndex};

#[cfg(feature = "postgres")]
pub(crate) fn postgres<'r, 'i, T>(
    row: &'r sqlx::postgres::PgRow,
    index: RowIndex<'i>,
) -> Result<T, RowError<'i>>
where
    T: sqlx::Decode<'r, sqlx::Postgres> + Type<sqlx::Postgres>,
{
    let value = try_get_raw(row, index)?;

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

#[cfg(feature = "sqlite")]
pub(crate) fn sqlite<'r, 'i, T>(
    row: &'r sqlx::sqlite::SqliteRow,
    index: RowIndex<'i>,
) -> Result<T, RowError<'i>>
where
    T: sqlx::Decode<'r, sqlx::Sqlite> + Type<sqlx::Sqlite>,
{
    let value = try_get_raw(row, index)?;

    if !value.is_null() {
        let ty = value.type_info();

        if !ty.is_null() && !T::compatible(&ty) {
            return Err(RowError::MismatchedTypes {
                index,
                rust_type: type_name::<T>(),
            });
        }
    } else if !is_option::<T>() {
        // sqlx' sqlite decoders don't check for `NULL` and would silently produce `0`, `""`, etc.
        return Err(RowError::UnexpectedNull { index });
    }

    T::decode(value).map_err(|source| {
        if source.is::<sqlx::error::UnexpectedNullError>() {
            RowError::UnexpectedNull { index }
        } else {
            RowError::Decode { index, source }
        }
    })
}

/// Checks whether `T` is an `Option<_>`
///
/// `Option<_>` is sqlx' only [`Decode`](sqlx::Decode) implementation which accepts `NULL`.
///
/// This implementation is a hack.
/// sqlx would allow to check this through `T::compatible` with a `NULL` type info,
/// but it doesn't provide a public way to construct such a `SqliteTypeInfo`.
#[cfg(feature = "sqlite")]
fn is_option<T>() -> bool {
    let option = type_name::<Option<()>>();
    let prefix = option.strip_suffix("()>").unwrap_or(option);
    type_name::<T>().starts_with(prefix)
}

fn try_get_raw<'r, 'i, D, R>(
    row: &'r R,
    index: RowIndex<'i>,
) -> Result<D::ValueRef<'r>, RowError<'i>>
where
    D: sqlx::Database,
    R: sqlx::Row<Database = D>,
    usize: ColumnIndex<R>,
    &'i str: ColumnIndex<R>,
{
    match index {
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
    })
}
