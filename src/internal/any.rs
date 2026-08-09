//! Simple alternative to [`sqlx::any`] which is tailored for rorm at the loss of generality.
//!
//! **Beware:**
//! `AnyQuery<'q>` and `AnyExecutor<'exe>` work quite different than `Query<'q, Any, _>` and `Executor<'e, Database = Any>`

use std::future::poll_fn;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::time::Duration;

use futures_core::Stream;
use rorm_declaration::config::DatabaseDriver;
use sqlx::query::Query;
#[cfg(feature = "postgres")]
use sqlx::{postgres, Postgres};
#[cfg(feature = "sqlite")]
use sqlx::{sqlite, Sqlite};
use sqlx::{ConnectOptions, Executor, Pool, SqlStr, Transaction};
use tracing::log::LevelFilter;

use crate::futures_util::BoxStream;
use crate::{DatabaseConfiguration, Error};

/// Enum around [`Pool<DB>`]
#[derive(Clone, Debug)]
pub enum AnyPool {
    #[cfg(feature = "postgres")]
    Postgres(Pool<Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Pool<Sqlite>),
}

impl AnyPool {
    /// Opens a pool of connections to a database
    pub async fn connect(configuration: DatabaseConfiguration) -> Result<AnyPool, Error> {
        /// All statements that take longer to execute than this value are considered
        /// as slow statements.
        const SLOW_STATEMENTS: Duration = Duration::from_millis(300);

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

        let pool: AnyPool = match &configuration.driver {
            #[cfg(feature = "sqlite")]
            DatabaseDriver::SQLite { filename } => {
                if filename.is_empty() {
                    return Err(Error::ConfigurationError(String::from(
                        "filename must not be empty",
                    )));
                }
                let connect_options = sqlite::SqliteConnectOptions::new()
                    .create_if_missing(true)
                    .filename(filename)
                    .log_slow_statements(LevelFilter::Warn, SLOW_STATEMENTS);
                AnyPool::Sqlite(
                    sqlite::SqlitePoolOptions::new()
                        .min_connections(configuration.min_connections)
                        .max_connections(configuration.max_connections)
                        .connect_with(connect_options)
                        .await?,
                )
            }
            #[cfg(feature = "postgres")]
            DatabaseDriver::Postgres {
                host,
                port,
                name,
                user,
                password,
            } => {
                if name.is_empty() {
                    return Err(Error::ConfigurationError(String::from(
                        "name must not be empty",
                    )));
                }
                let connect_options = postgres::PgConnectOptions::new()
                    .host(host.as_str())
                    .port(*port)
                    .username(user.as_str())
                    .password(password.as_str())
                    .database(name.as_str())
                    .log_slow_statements(LevelFilter::Warn, SLOW_STATEMENTS);
                AnyPool::Postgres(
                    postgres::PgPoolOptions::new()
                        .min_connections(configuration.min_connections)
                        .max_connections(configuration.max_connections)
                        .connect_with(connect_options)
                        .await?,
                )
            }
        };

        Ok(pool)
    }

    /// Retrieves a connection and immediately begins a new transaction.
    ///
    /// See [`Pool::begin`]
    pub async fn begin(&self) -> sqlx::Result<AnyTransaction> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(pool) => pool.begin().await.map(AnyTransaction::Postgres),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(pool) => pool.begin().await.map(AnyTransaction::Sqlite),
        }
    }

    /// Shut down the connection pool, immediately waking all tasks waiting for a connection.
    ///
    /// See [`Pool::close`]
    pub async fn close(&self) {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(pool) => pool.close().await,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(pool) => pool.close().await,
        }
    }
    /// Returns `true` if [.close()](Self::close) has been called on the pool, `false` otherwise.
    ///
    /// See [`Pool::is_closed`]
    pub fn is_closed(&self) -> bool {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(pool) => pool.is_closed(),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(pool) => pool.is_closed(),
        }
    }
}

/// Enum around [`Transaction<'static, DB>`]
pub enum AnyTransaction {
    #[cfg(feature = "postgres")]
    Postgres(Transaction<'static, Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Transaction<'static, Sqlite>),
}

impl AnyTransaction {
    /// Commits this transaction or savepoint.
    ///
    /// See [Transaction::commit]
    pub async fn commit(self) -> sqlx::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(tx) => tx.commit().await,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(tx) => tx.commit().await,
        }
    }

    /// Aborts this transaction or savepoint.
    ///
    /// See [Transaction::rollback]
    pub async fn rollback(self) -> sqlx::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(tx) => tx.rollback().await,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(tx) => tx.rollback().await,
        }
    }
}

/// Combination of an [`AnyExecutor`] and its associated [`Query<'q, DB, _>`]
pub enum AnyQuery<'exe> {
    #[cfg(feature = "postgres")]
    PostgresPool {
        executor: &'exe Pool<Postgres>,
        query: Option<Query<'static, Postgres, postgres::PgArguments>>,
    },
    #[cfg(feature = "sqlite")]
    SqlitePool {
        executor: &'exe Pool<Sqlite>,
        query: Option<Query<'static, Sqlite, sqlite::SqliteArguments>>,
    },
    #[cfg(feature = "postgres")]
    PostgresConn {
        executor: &'exe mut postgres::PgConnection,
        query: Option<Query<'static, Postgres, postgres::PgArguments>>,
    },
    #[cfg(feature = "sqlite")]
    SqliteConn {
        executor: &'exe mut sqlite::SqliteConnection,
        query: Option<Query<'static, Sqlite, sqlite::SqliteArguments>>,
    },
}

impl<'exe> AnyQuery<'exe> {
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`]
    pub fn bind<'t, T>(&mut self, value: T)
    where
        T: Send + AnyEncode<'t> + AnyType,
    {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool { query, .. } | Self::PostgresConn { query, .. } => {
                *query = query.take().map(|query| query.bind(value))
            }
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { query, .. } | Self::SqliteConn { query, .. } => {
                *query = query.take().map(|query| query.bind(value))
            }
        }
    }

    /// Execute the query and return the generated results in a stream.
    pub fn fetch_many(self) -> BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>> {
        struct MappedStream<'stream, LI, LM, RI, RM> {
            stream: BoxStream<'stream, sqlx::Result<sqlx::Either<LI, RI>>>,
            map_left: LM,
            map_right: RM,
        }
        impl<LI, LM, RI, RM> Unpin for MappedStream<'_, LI, LM, RI, RM> {}
        impl<LI, LM, RI, RM> Stream for MappedStream<'_, LI, LM, RI, RM>
        where
            LM: Fn(LI) -> AnyQueryResult,
            RM: Fn(RI) -> AnyRow,
        {
            type Item = sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                let opt = ready!(self.stream.as_mut().poll_next(cx));
                Poll::Ready(opt.map(|res| {
                    res.map(|either| either.map_left(&self.map_left).map_right(&self.map_right))
                }))
            }
        }
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool { executor, query } => Box::pin(MappedStream {
                stream: executor.fetch_many(query.unwrap()),
                map_left: AnyQueryResult::Postgres,
                map_right: AnyRow::Postgres,
            }),
            #[cfg(feature = "postgres")]
            Self::PostgresConn { executor, query } => Box::pin(MappedStream {
                stream: executor.fetch_many(query.unwrap()),
                map_left: AnyQueryResult::Postgres,
                map_right: AnyRow::Postgres,
            }),
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { executor, query } => Box::pin(MappedStream {
                stream: executor.fetch_many(query.unwrap()),
                map_left: AnyQueryResult::Sqlite,
                map_right: AnyRow::Sqlite,
            }),
            #[cfg(feature = "sqlite")]
            Self::SqliteConn { executor, query } => Box::pin(MappedStream {
                stream: executor.fetch_many(query.unwrap()),
                map_left: AnyQueryResult::Sqlite,
                map_right: AnyRow::Sqlite,
            }),
        }
    }

    /// Execute the query and return all the generated results, collected into a `Vec`.
    pub async fn fetch_all(self) -> sqlx::Result<Vec<AnyRow>> {
        let mut vec = Vec::new();
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    if let Some(row) = either.right() {
                        vec.push(AnyRow::Postgres(row));
                    }
                }
            }
            #[cfg(feature = "postgres")]
            Self::PostgresConn { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    if let Some(row) = either.right() {
                        vec.push(AnyRow::Postgres(row));
                    }
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    if let Some(row) = either.right() {
                        vec.push(AnyRow::Sqlite(row));
                    }
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqliteConn { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    if let Some(row) = either.right() {
                        vec.push(AnyRow::Sqlite(row));
                    }
                }
            }
        }
        Ok(vec)
    }

    /// Execute the query and returns at most one row.
    pub async fn fetch_optional(self) -> sqlx::Result<Option<AnyRow>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool { executor, query } => executor
                .fetch_optional(query.unwrap())
                .await
                .map(|option| option.map(AnyRow::Postgres)),
            #[cfg(feature = "postgres")]
            Self::PostgresConn { executor, query } => executor
                .fetch_optional(query.unwrap())
                .await
                .map(|option| option.map(AnyRow::Postgres)),
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { executor, query } => executor
                .fetch_optional(query.unwrap())
                .await
                .map(|option| option.map(AnyRow::Sqlite)),
            #[cfg(feature = "sqlite")]
            Self::SqliteConn { executor, query } => executor
                .fetch_optional(query.unwrap())
                .await
                .map(|option| option.map(AnyRow::Sqlite)),
        }
    }

    /// Execute the query and return the number of affected rows.
    pub async fn fetch_affected_rows(self) -> sqlx::Result<u64> {
        let mut count = 0;
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    match either {
                        sqlx::Either::Left(result) => count += result.rows_affected(),
                        sqlx::Either::Right(_row) => {}
                    }
                }
            }
            #[cfg(feature = "postgres")]
            Self::PostgresConn { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    match either {
                        sqlx::Either::Left(result) => count += result.rows_affected(),
                        sqlx::Either::Right(_row) => {}
                    }
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    match either {
                        sqlx::Either::Left(result) => count += result.rows_affected(),
                        sqlx::Either::Right(_row) => {}
                    }
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqliteConn { executor, query } => {
                let mut stream = executor.fetch_many(query.unwrap());
                while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
                    .await
                    .transpose()?
                {
                    match either {
                        sqlx::Either::Left(result) => count += result.rows_affected(),
                        sqlx::Either::Right(_row) => {}
                    }
                }
            }
        }
        Ok(count)
    }
}

/// Enum around [`<DB as Database>::Row`](Database#associatedtype.Row)
pub enum AnyRow {
    #[cfg(feature = "postgres")]
    Postgres(postgres::PgRow),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlite::SqliteRow),
}

/// Enum around [`<DB as Database>::QueryResult`](Database#associatedtype.Row)
pub enum AnyQueryResult {
    #[cfg(feature = "postgres")]
    Postgres(postgres::PgQueryResult),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlite::SqliteQueryResult),
}

impl AnyQueryResult {
    /// The number of rows affected by a query
    pub fn rows_affected(&self) -> u64 {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(result) => result.rows_affected(),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(result) => result.rows_affected(),
        }
    }
}

/// Trait to start queries from either an [`AnyPool`] or an [`AnyTransaction`]
pub trait AnyExecutor<'exe> {
    /// Start a query
    ///
    /// This will consume the executor and store it alongside the query until it is executed.
    // This way there is no additional check required whether the executor's db matches the query's
    fn query(self, query: SqlStr) -> AnyQuery<'exe>;
}
impl<'exe> AnyExecutor<'exe> for &'exe AnyPool {
    fn query(self, query: SqlStr) -> AnyQuery<'exe> {
        match self {
            #[cfg(feature = "postgres")]
            AnyPool::Postgres(pool) => AnyQuery::PostgresPool {
                executor: pool,
                query: Some(sqlx::query(query)),
            },
            #[cfg(feature = "sqlite")]
            AnyPool::Sqlite(pool) => AnyQuery::SqlitePool {
                executor: pool,
                query: Some(sqlx::query(query)),
            },
        }
    }
}
impl<'exe> AnyExecutor<'exe> for &'exe mut AnyTransaction {
    fn query(self, query: SqlStr) -> AnyQuery<'exe> {
        match self {
            #[cfg(feature = "postgres")]
            AnyTransaction::Postgres(tx) => AnyQuery::PostgresConn {
                executor: tx.deref_mut(),
                query: Some(sqlx::query(query)),
            },
            #[cfg(feature = "sqlite")]
            AnyTransaction::Sqlite(tx) => AnyQuery::SqliteConn {
                executor: tx.deref_mut(),
                query: Some(sqlx::query(query)),
            },
        }
    }
}

macro_rules! uncond_trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $($bound:path,)+) => {
        $(#[doc = $doc])*
        pub trait $trait $(<$lifetime>)?
        where
            $(Self: $bound),+
        {}

        impl<$($lifetime,)? T> $trait $(<$lifetime>)? for T
        where
            $(Self: $bound),+
        {}
    };
}

#[rustfmt::skip]
#[cfg(all(feature = "postgres", feature = "sqlite"))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $postgres, $sqlite,);
    };
}

#[rustfmt::skip]
#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $sqlite,);
    };
}

#[rustfmt::skip]
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
macro_rules! trait_alias {
    ($(#[doc = $doc:literal])* trait $trait:ident $(<$lifetime:lifetime>)?: $postgres:path, $sqlite:path,) => {
        uncond_trait_alias!($(#[doc = $doc])* trait $trait $(<$lifetime>)?: $postgres,);
    };
}

trait_alias!(
    /// Trait alias combining all [`Encode<'q, DB>`](sqlx::Encode)
    trait AnyEncode<'q>: sqlx::Encode<'q, Postgres>, sqlx::Encode<'q, Sqlite>,
);

trait_alias!(
    /// Trait alias combining all [`Decode<'r, DB>`](sqlx::Decode)
    trait AnyDecode<'r>: sqlx::Decode<'r, Postgres>, sqlx::Decode<'r, Sqlite>,
);

trait_alias!(
    /// Trait alias combining all [`Type<DB>`](sqlx::Type)
    trait AnyType: sqlx::Type<Postgres>, sqlx::Type<Sqlite>,
);
