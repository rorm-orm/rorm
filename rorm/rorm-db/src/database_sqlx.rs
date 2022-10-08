/*!
This module defines the main API wrapper.
*/

use futures::stream::BoxStream;
use futures::StreamExt;
use rorm_sql::{conditional, value, DBImpl};
use sqlx::any::AnyPoolOptions;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::sqlite::SqliteConnectOptions;

use crate::error::Error;
use crate::result::QueryStream;
use crate::row::Row;
use crate::{utils, DatabaseBackend, DatabaseConfiguration};

/**
Main API wrapper.

All operations can be started with methods of this struct.
 */
pub struct Database {
    pool: sqlx::Pool<sqlx::Any>,
    db_impl: DBImpl,
}

impl Database {
    /**
    Connect to the database using `configuration`.
     */
    pub async fn connect(configuration: DatabaseConfiguration) -> Result<Self, Error> {
        if configuration.max_connections < configuration.min_connections {
            return Err(Error::ConfigurationError(String::from(
                "max_connections must not be less than min_connections",
            )));
        }

        if configuration.min_connections == 0 {
            return Err(Error::ConfigurationError(String::from(
                "min_connections must not be 0",
            )));
        }

        if configuration.name == "" {
            return Err(Error::ConfigurationError(String::from(
                "name must not be empty",
            )));
        }

        let database;
        let pool_options = AnyPoolOptions::new()
            .min_connections(configuration.min_connections)
            .max_connections(configuration.max_connections);

        let pool;

        match configuration.backend {
            DatabaseBackend::SQLite => {
                let connect_options = SqliteConnectOptions::new()
                    .create_if_missing(true)
                    .filename(configuration.name);
                pool = pool_options.connect_with(connect_options.into()).await?;
            }
            DatabaseBackend::Postgres => {
                let connect_options = PgConnectOptions::new()
                    .host(configuration.host.as_str())
                    .port(configuration.port)
                    .username(configuration.user.as_str())
                    .password(configuration.password.as_str())
                    .database(configuration.name.as_str());
                pool = pool_options.connect_with(connect_options.into()).await?;
            }
            DatabaseBackend::MySQL => {
                let connect_options = MySqlConnectOptions::new()
                    .host(configuration.host.as_str())
                    .port(configuration.port)
                    .username(configuration.user.as_str())
                    .password(configuration.password.as_str())
                    .database(configuration.name.as_str());
                pool = pool_options.connect_with(connect_options.into()).await?;
            }
        }

        database = Database {
            pool,
            db_impl: match configuration.backend {
                DatabaseBackend::SQLite => DBImpl::SQLite,
                DatabaseBackend::Postgres => DBImpl::Postgres,
                DatabaseBackend::MySQL => DBImpl::MySQL,
            },
        };

        return Ok(database);
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
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'post_query>>,
    ) -> BoxStream<'stream, Result<Row, Error>>
    where
        'post_query: 'stream,
        'db: 'stream,
    {
        let mut q = self.db_impl.select(columns, model);
        if conditions.is_some() {
            q = q.where_clause(conditions.unwrap());
        }

        let (query_string, bind_params) = q.build();

        return QueryStream::build(query_string, bind_params, &self.pool).boxed();
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
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Row, Error> {
        let mut q = self.db_impl.select(columns, model);
        if conditions.is_some() {
            q = q.where_clause(conditions.unwrap());
        }

        let (query_string, bind_params) = q.build();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        tmp.fetch_one(&self.pool)
            .await
            .map(Row::from)
            .map_err(Error::SqlxError)
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
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Option<Row>, Error> {
        let mut q = self.db_impl.select(columns, model);
        if conditions.is_some() {
            q = q.where_clause(conditions.unwrap());
        }

        let (query_string, bind_params) = q.build();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        tmp.fetch_optional(&self.pool)
            .await
            .map(|option| option.map(Row::from))
            .map_err(Error::SqlxError)
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
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Vec<Row>, Error> {
        let mut q = self.db_impl.select(columns, model);
        if conditions.is_some() {
            q = q.where_clause(conditions.unwrap());
        }

        let (query_string, bind_params) = q.build();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        tmp.fetch_all(&self.pool)
            .await
            .map(|vector| vector.into_iter().map(Row::from).collect())
            .map_err(Error::SqlxError)
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
        model: &str,
        columns: &[&str],
        values: &[value::Value<'_>],
    ) -> Result<(), Error> {
        let value_rows = &[values];
        let q = self.db_impl.insert(model, columns, value_rows);

        let (query_string, bind_params) = q.build();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        match tmp.execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::SqlxError(err)),
        }
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
        model: &str,
        columns: &[&str],
        rows: &[&[value::Value<'_>]],
    ) -> Result<(), Error> {
        let mut t = self.db_impl.start_transaction();

        let mut bind_params = vec![];
        for chunk in rows.chunks(25) {
            let mut q = self.db_impl.insert(model, columns, chunk);
            q = q.rollback_transaction();
            let (insert_query, insert_params) = q.build();
            t = t.add_statement(insert_query);
            bind_params.extend(insert_params);
        }

        let query_string = t.finish();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        match tmp.execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::SqlxError(err)),
        }
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
        model: &str,
        condition: Option<&conditional::Condition<'post_build>>,
    ) -> Result<u64, Error> {
        let mut q = self.db_impl.delete(model);
        if condition.is_some() {
            q = q.where_clause(condition.unwrap());
        }

        let (query_string, bind_params) = q.build();

        let mut tmp = sqlx::query(query_string.as_str());
        for x in bind_params {
            tmp = utils::bind_param(tmp, x);
        }

        match tmp.execute(&self.pool).await {
            Ok(qr) => Ok(qr.rows_affected()),
            Err(err) => Err(Error::SqlxError(err)),
        }
    }
}
