//! Query builder and macro
use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures::StreamExt;
use rorm_db::{conditional::Condition, error::Error, row::Row, Database};

use crate::crud::builder::ConditionMarker;
use crate::model::{Model, Patch};

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
pub struct QueryBuilder<'a, P: Patch, C: ConditionMarker<'a>> {
    db: &'a Database,
    columns: &'a [&'a str],
    _phantom: PhantomData<P>,

    condition: C,
}

impl<'a, P: Patch> QueryBuilder<'a, P, ()> {
    /// Start building a query which selects a given set of columns
    pub fn select_columns(db: &'a Database, columns: &'a [&'a str]) -> Self {
        QueryBuilder {
            db,
            columns, // TODO: check for existence
            _phantom: PhantomData,

            condition: (),
        }
    }

    /// Start building a query which selects a patch's columns
    pub fn select_patch(db: &'a Database) -> Self {
        Self::select_columns(db, P::COLUMNS)
    }
}

impl<'a, P: Patch> QueryBuilder<'a, P, ()> {
    /// Add a condition to the query
    pub fn condition(&self, condition: Condition<'a>) -> QueryBuilder<'a, P, Condition<'a>> {
        QueryBuilder {
            db: self.db,
            columns: self.columns,
            _phantom: self._phantom,

            condition,
        }
    }
}

impl<'a, P: Patch + Send + 'static, C: ConditionMarker<'a>> QueryBuilder<'a, P, C> {
    /// Retrieve the query as a stream of Models
    pub fn stream(&self) -> BoxStream<'a, Result<P, Error>> {
        self.stream_as_row()
            .map(|row| {
                P::try_from(row?)
                    .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
            })
            .boxed()
    }
}

impl<'a, P: Patch, C: ConditionMarker<'a>> QueryBuilder<'a, P, C> {
    /// Retrieve all matching rows as unpacked models
    pub async fn all(&self) -> Result<Vec<P>, Error> {
        self.all_as_rows()
            .await?
            .into_iter()
            .map(|x| {
                P::try_from(x).map_err(|_| Error::DecodeError("Could not decode row".to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// Retrieve all matching rows
    pub async fn all_as_rows(&self) -> Result<Vec<Row>, Error> {
        self.db
            .query_all(
                P::Model::table_name(),
                self.columns,
                self.condition.as_option(),
            )
            .await
    }

    /// Retrieve exactly one matching row as Model
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one(&self) -> Result<P, Error> {
        P::try_from(self.one_as_row().await?)
            .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
    }

    /// Retrieve exactly one matching row
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one_as_row(&self) -> Result<Row, Error> {
        self.db
            .query_one(
                P::Model::table_name(),
                self.columns,
                self.condition.as_option(),
            )
            .await
    }

    /// Retrieve the query as a stream of rows
    pub fn stream_as_row(&self) -> BoxStream<'a, Result<Row, Error>> {
        self.db.query_stream(
            P::Model::table_name(),
            self.columns,
            self.condition.as_option(),
        )
    }

    /// Try to retrieve the a matching row as Model
    pub async fn optional(&self) -> Result<Option<P>, Error> {
        match self.optional_as_row().await? {
            None => Ok(None),
            Some(row) => {
                Ok(Some(P::try_from(row).map_err(|_| {
                    Error::DecodeError("Could not decode row".to_string())
                })?))
            }
        }
    }

    /// Try to retrieve the a matching row
    pub async fn optional_as_row(&self) -> Result<Option<Row>, Error> {
        self.db
            .query_optional(
                P::Model::table_name(),
                self.columns,
                self.condition.as_option(),
            )
            .await
    }
}

/// Slightly less verbose macro to start a [`QueryBuilder`] from a model or patch
#[macro_export]
macro_rules! query {
    ($db:expr, $patch:path) => {
        $crate::QueryBuilder::select_patch::<$patch>(&$db)
    };
}
