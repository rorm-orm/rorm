use crate::model::{Model, Patch};
use futures::stream::BoxStream;
use rorm_db::{error::Error, row::Row, Database};
use std::marker::PhantomData;

/// Builder for creating queries
///
/// 1. Start by specifying which columns to select:
///     - [`QueryBuilder::select_columns`] to select columns manually
///     - [`QueryBuilder::select_model`] to select all columns the model has
///     - [`QueryBuilder::select_patch`] to select all columns a patch has
/// 2. Add optional modifiers:
///     - [`QueryBuilder::condition`]
/// 3. Retrieve rows from the db:
///     - [`QueryBuilder::all`] to get all rows at once
///     - [`QueryBuilder::stream`] to get all rows in a stream
///     - [`QueryBuilder::one`] to get a single row
///     - [`QueryBuilder::optional`] to get an optional single row
pub struct QueryBuilder<'a, M: Model, C: ConditionMarker> {
    db: &'a Database,
    columns: &'a [&'a str],
    _phantom: PhantomData<*const M>,

    condition: C,
}

impl<'a, M: Model> QueryBuilder<'a, M, ()> {
    /// Start building a query which selects a given set of columns
    pub fn select_columns(db: &'a Database, columns: &'a [&'a str]) -> Self {
        QueryBuilder {
            db,
            columns, // TODO: check for existence
            _phantom: PhantomData,

            condition: (),
        }
    }

    /// Start building a query which selects every column
    pub fn select_model(db: &'a Database) -> Self
    where
        M: Patch<MODEL = M>,
    {
        Self::select_patch::<M>(db)
    }

    /// Start building a query which selects a patch's columns
    pub fn select_patch<P: Patch<MODEL = M>>(db: &'a Database) -> Self {
        Self::select_columns(db, P::COLUMNS)
    }
}

impl<'a, M: Model> QueryBuilder<'a, M, ()> {
    /// Add a condition to the query
    pub fn condition(&self, condition: Condition<'a>) -> QueryBuilder<'a, M, Condition> {
        QueryBuilder {
            db: self.db,
            columns: self.columns,
            _phantom: self._phantom,

            condition,
        }
    }
}

impl<'a, M: Model, C: ConditionMarker> QueryBuilder<'a, M, C> {
    /// Retrieve all matching rows
    pub async fn all(&self) -> Result<Vec<Row>, Error> {
        self.db
            .query_all(M::table_name(), self.columns, self.condition.as_option())
            .await
    }

    /// Retrieve exactly one matching row
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one(&self) -> Result<Row, Error> {
        self.db
            .query_one(M::table_name(), self.columns, self.condition.as_option())
            .await
    }

    /// Retrieve the query as a stream of rows
    pub async fn stream(&self) -> BoxStream<'_, Result<Row, Error>> {
        self.db
            .query_stream(M::table_name(), self.columns, self.condition.as_option())
    }

    /// Try to retrieve the a matching row
    pub async fn optional(&self) -> Result<Option<Row>, Error> {
        self.db
            .query_optional(M::table_name(), self.columns, self.condition.as_option())
            .await
    }
}

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
use private::Private;
use rorm_db::conditional::Condition;

#[doc(hidden)]
pub trait ConditionMarker {
    fn __private<P: Private>() {}

    fn as_option(&self) -> Option<&Condition>;
}

impl ConditionMarker for () {
    fn __private<P: Private>() {}

    fn as_option(&self) -> Option<&Condition> {
        None
    }
}

impl ConditionMarker for Condition<'_> {
    fn __private<P: Private>() {}

    fn as_option(&self) -> Option<&Condition> {
        Some(self)
    }
}
