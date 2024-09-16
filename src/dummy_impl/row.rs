use super::{no_sqlx, NotInstantiable};
use crate::row::{Decode, RowError, RowIndex};
use crate::{Error, Row};

pub(crate) type Impl = NotInstantiable;

/// Implementation of [Row::get]
pub(crate) fn get<'r, 'i, T>(row: &'r Row, index: RowIndex<'i>) -> Result<T, RowError<'i>>
where
    T: Decode<'r>,
{
    no_sqlx();
}
