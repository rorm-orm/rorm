/*!
This module defines the main API wrapper.
 */

use std::marker::PhantomData;

use futures::stream::BoxStream;
use rorm_sql::{conditional, value};

use crate::error::Error;
use crate::row::Row;
use crate::DatabaseConfiguration;

/**
Main API wrapper.

All operations can be started with methods of this struct.
 */
pub struct Database(PhantomData<()>);

impl Database {
    /**
    Connect to the database using `configuration`.
     */
    pub async fn connect(_configuration: DatabaseConfiguration) -> Result<Self, Error> {
        Err(Error::ConfigurationError(
            "Can't work with the database without sqlx".to_string(),
        ))
    }

    /**
    This method is used to retrieve a stream of rows that matched the applied conditions.

    **Parameter**:
    - `model`: Name of the table.
    - `columns`: Columns to retrieve values from.
    - `conditions`: Optional conditions to apply.
     */
    pub fn query_stream<'db, 'post_query, 'stream>(
        &'db self,
        _model: &str,
        _columns: &[&str],
        _conditions: Option<&conditional::Condition<'post_query>>,
    ) -> BoxStream<'stream, Result<Row, Error>>
    where
        'post_query: 'stream,
        'db: 'stream,
    {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to retrieve exactly one row from the table.
    An error is returned if no value could be retrieved.

    **Parameter**:
    - `model`: Model to query.
    - `columns`: Columns to retrieve values from.
    - `conditions`: Optional conditions to apply.
     */
    pub async fn query_one(
        &self,
        _model: &str,
        _columns: &[&str],
        _conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Row, Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to retrieve an optional row from the table.

    **Parameter**:
    - `model`: Model to query.
    - `columns`: Columns to retrieve values from.
    - `conditions`: Optional conditions to apply.
     */
    pub async fn query_optional(
        &self,
        _model: &str,
        _columns: &[&str],
        _conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Option<Row>, Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to retrieve all rows that match the provided query.

    **Parameter**:
    - `model`: Model to query.
    - `columns`: Columns to retrieve values from.
    - `conditions`: Optional conditions to apply.
     */
    pub async fn query_all(
        &self,
        _model: &str,
        _columns: &[&str],
        _conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Vec<Row>, Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to insert into a table.

    **Parameter**:
    - `model`: Table to insert to
    - `columns`: Columns to set `values` for.
    - `values`: Values to bind to the corresponding columns.
     */
    pub async fn insert(
        &self,
        _model: &str,
        _columns: &[&str],
        _values: &[value::Value<'_>],
    ) -> Result<(), Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to bulk insert rows.

    If one insert statement fails, the complete operation will be rolled back.

    **Parameter**:
    - `model`: Table to insert to
    - `columns`: Columns to set `rows` for.
    - `rows`: List of values to bind to the corresponding columns.
     */
    pub async fn insert_bulk(
        &self,
        _model: &str,
        _columns: &[&str],
        _rows: &[&[value::Value<'_>]],
    ) -> Result<(), Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }

    /**
    This method is used to delete rows from a table.

    **Parameter**:
    - `model`: Name of the model to delete rows from
    - `condition`: Optional condition to apply.

    **Returns** the rows affected of the delete statement. Note that this also includes
    relations, etc.
     */
    pub async fn delete<'post_build>(
        &self,
        _model: &str,
        _condition: Option<&conditional::Condition<'post_build>>,
    ) -> Result<u64, Error> {
        unreachable!("connect shouldn't give you an instance to call this method on, without sqlx");
    }
}
