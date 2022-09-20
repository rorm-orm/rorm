//! This crate is used as language independent base for building an orm.
//!
//! Rust specific features will be exposed through the `rorm` crate.
//! `rorm-lib` implements C bindings for this crate.
#![warn(missing_docs)]

/**
Errors of rorm-db will be specified here.
*/
pub mod error;

#[cfg(feature = "sqlx-dep")]
/**
This module holds the definitions of queries and their results
*/
pub mod query;

#[cfg(feature = "sqlx-dep")]
/**
This module holds the results of a query
*/
pub mod result;
#[cfg(feature = "sqlx-dep")]
/**
This module defines a wrapper for sqlx's AnyRow
*/
pub mod row;
#[cfg(feature = "sqlx-dep")]
mod utils;

use futures::stream::BoxStream;
use futures::StreamExt;
use rorm_sql::DBImpl;
#[cfg(feature = "sqlx-dep")]
use sqlx::any::AnyPoolOptions;
#[cfg(feature = "sqlx-dep")]
use sqlx::mysql::MySqlConnectOptions;
#[cfg(feature = "sqlx-dep")]
use sqlx::postgres::PgConnectOptions;
#[cfg(feature = "sqlx-dep")]
use sqlx::sqlite::SqliteConnectOptions;

pub use rorm_sql::conditional;
pub use rorm_sql::value;
pub use rorm_sql::{and, or};

use crate::error::Error;
#[cfg(feature = "sqlx-dep")]
use crate::result::QueryStream;

/**
Representation of different backends
*/
pub enum DatabaseBackend {
    /// SQLite database backend
    SQLite,
    /// Postgres database backend
    Postgres,
    /// MySQL / MariaDB database backend
    MySQL,
}

/**
Configuration to create a database connection.

If [DatabaseBackend::SQLite] is used as backend, `name` specifies the filename.
`host`, `port`, `user`, `password` is not used in this case.

If [DatabaseBackend::Postgres] or [DatabaseBackend::MySQL] is used, `name` specifies the
database to connect to.

`min_connections` and `max_connections` must be greater than 0
and `max_connections` must be greater or equals `min_connections`.
*/
pub struct DatabaseConfiguration {
    /// Specifies the driver that will be used
    pub backend: DatabaseBackend,
    /// Name of the database, in case of [DatabaseBackend::SQLite] name of the file.
    pub name: String,
    /// Host to connect to. Not used in case of [DatabaseBackend::SQLite].
    pub host: String,
    /// Port to connect to. Not used in case of [DatabaseBackend::SQLite].
    pub port: u16,
    /// Username to authenticate with. Not used in case of [DatabaseBackend::SQLite].
    pub user: String,
    /// Password to authenticate with. Not used in case of [DatabaseBackend::SQLite].
    pub password: String,
    /// Minimal connections to initialize upfront.
    pub min_connections: u32,
    /// Maximum connections that allowed to be created.
    pub max_connections: u32,
}

#[cfg(feature = "sqlx-dep")]
/**
Main API wrapper.

All operations can be started with methods of this struct.
*/
pub struct Database {
    pool: sqlx::Pool<sqlx::Any>,
    db_impl: DBImpl,
}

#[cfg(feature = "sqlx-dep")]
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

    `model`: Name of the table.
    `columns`: Columns to retrieve values from.
    `conditions`: Optional conditions to apply.
    */
    pub fn query_stream<'db, 'post_query, 'stream>(
        &'db self,
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'post_query>>,
    ) -> BoxStream<'stream, Result<row::Row, Error>>
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

    `model`: Model to query.
    `columns`: Columns to retrieve values from.
    `conditions`: Optional conditions to apply.
    */
    pub async fn query_one(
        &self,
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<row::Row, Error> {
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
            .map(row::Row::from)
            .map_err(Error::SqlxError)
    }

    /**
    This method is used to retrieve an optional row from the table.

    `model`: Model to query.
    `columns`: Columns to retrieve values from.
    `conditions`: Optional conditions to apply.
    */
    pub async fn query_optional(
        &self,
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Option<row::Row>, Error> {
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
            .map(|option| option.map(row::Row::from))
            .map_err(Error::SqlxError)
    }

    /**
    This method is used to retrieve all rows that match the provided query.

    `model`: Model to query.
    `columns`: Columns to retrieve values from.
    `conditions`: Optional conditions to apply.
    */
    pub async fn query_all(
        &self,
        model: &str,
        columns: &[&str],
        conditions: Option<&conditional::Condition<'_>>,
    ) -> Result<Vec<row::Row>, Error> {
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
            .map(|vector| vector.into_iter().map(row::Row::from).collect())
            .map_err(Error::SqlxError)
    }
}