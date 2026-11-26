//! Simple alternative to [`sqlx::any`] which is tailored for rorm at the loss of generality.
//!
//! **Beware:**
//! `AnyQuery<'q>` and `AnyExecutor<'exe>` work quite different than `Query<'q, Any, _>` and `Executor<'e, Database = Any>`

use std::future::Future;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::time::Duration;

use crate::futures_util::{BoxFuture, BoxStream};
use crate::internal::query_span::QuerySpanBuilder;
use crate::{DatabaseConfiguration, Error};
use futures_core::Stream;
use rorm_declaration::config::DatabaseDriver;
use sqlx::query::Query;
#[cfg(feature = "postgres")]
use sqlx::{postgres, Postgres};
#[cfg(feature = "sqlite")]
use sqlx::{sqlite, Sqlite};
use sqlx::{ConnectOptions, Executor, Pool, SqlStr, Transaction};
use tracing::log::LevelFilter;
use tracing::Span;

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
        span: QuerySpanBuilder,
    },
    #[cfg(feature = "sqlite")]
    SqlitePool {
        executor: &'exe Pool<Sqlite>,
        query: Option<Query<'static, Sqlite, sqlite::SqliteArguments>>,
        span: QuerySpanBuilder,
    },
    #[cfg(feature = "postgres")]
    PostgresConn {
        executor: &'exe mut postgres::PgConnection,
        query: Option<Query<'static, Postgres, postgres::PgArguments>>,
        span: QuerySpanBuilder,
    },
    #[cfg(feature = "sqlite")]
    SqliteConn {
        executor: &'exe mut sqlite::SqliteConnection,
        query: Option<Query<'static, Sqlite, sqlite::SqliteArguments>>,
        span: QuerySpanBuilder,
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
            Self::PostgresPool { query, span, .. } | Self::PostgresConn { query, span, .. } => {
                span.track_bind();
                *query = query.take().map(|query| query.bind(value))
            }
            #[cfg(feature = "sqlite")]
            Self::SqlitePool { query, span, .. } | Self::SqliteConn { query, span, .. } => {
                span.track_bind();
                *query = query.take().map(|query| query.bind(value))
            }
        }
    }

    /// Execute the query and return the generated results in a stream.
    pub fn fetch_many(self) -> AnyFetchMany<'exe> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool {
                executor,
                query,
                span,
            } => {
                let span = span.finish();
                AnyFetchMany::Postgres {
                    stream: span.in_scope(|| executor.fetch_many(query.unwrap())),
                    span,
                }
            }
            #[cfg(feature = "postgres")]
            Self::PostgresConn {
                executor,
                query,
                span,
            } => {
                let span = span.finish();
                AnyFetchMany::Postgres {
                    stream: span.in_scope(|| executor.fetch_many(query.unwrap())),
                    span,
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqlitePool {
                executor,
                query,
                span,
            } => {
                let span = span.finish();
                AnyFetchMany::Sqlite {
                    stream: span.in_scope(|| executor.fetch_many(query.unwrap())),
                    span,
                }
            }
            #[cfg(feature = "sqlite")]
            Self::SqliteConn {
                executor,
                query,
                span,
            } => {
                let span = span.finish();
                AnyFetchMany::Sqlite {
                    stream: span.in_scope(|| executor.fetch_many(query.unwrap())),
                    span,
                }
            }
        }
    }

    /// Execute the query and returns at most one row.
    pub fn fetch_optional(self) -> AnyFetchOptional<'exe> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresPool {
                executor,
                query,
                span,
            } => AnyFetchOptional::Postgres {
                future: executor.fetch_optional(query.unwrap()),
                span: span.finish(),
            },
            #[cfg(feature = "postgres")]
            Self::PostgresConn {
                executor,
                query,
                span,
            } => AnyFetchOptional::Postgres {
                future: executor.fetch_optional(query.unwrap()),
                span: span.finish(),
            },
            #[cfg(feature = "sqlite")]
            Self::SqlitePool {
                executor,
                query,
                span,
            } => AnyFetchOptional::Sqlite {
                future: executor.fetch_optional(query.unwrap()),
                span: span.finish(),
            },
            #[cfg(feature = "sqlite")]
            Self::SqliteConn {
                executor,
                query,
                span,
            } => AnyFetchOptional::Sqlite {
                future: executor.fetch_optional(query.unwrap()),
                span: span.finish(),
            },
        }
    }
}

/// [`Stream`] returned by [`AnyQuery::fetch_many`]
pub enum AnyFetchMany<'stream> {
    #[cfg(feature = "postgres")]
    Postgres {
        stream: BoxStream<
            'stream,
            sqlx::Result<sqlx::Either<postgres::PgQueryResult, postgres::PgRow>>,
        >,
        span: Span,
    },
    #[cfg(feature = "sqlite")]
    Sqlite {
        stream: BoxStream<
            'stream,
            sqlx::Result<sqlx::Either<sqlite::SqliteQueryResult, sqlite::SqliteRow>>,
        >,
        span: Span,
    },
}
impl Unpin for AnyFetchMany<'_> {}
impl Stream for AnyFetchMany<'_> {
    type Item = sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this {
            #[cfg(feature = "postgres")]
            Self::Postgres { stream, span } => {
                let opt = ready!(span.in_scope(|| stream.as_mut().poll_next(cx)));
                *span = Span::none(); // Drop span after first item
                Poll::Ready(opt.map(|res| {
                    res.map(|either| {
                        either
                            .map_left(AnyQueryResult::Postgres)
                            .map_right(AnyRow::Postgres)
                    })
                }))
            }
            #[cfg(feature = "sqlite")]
            Self::Sqlite { stream, span } => {
                let opt = ready!(span.in_scope(|| stream.as_mut().poll_next(cx)));
                *span = Span::none(); // Drop span after first item
                Poll::Ready(opt.map(|res| {
                    res.map(|either| {
                        either
                            .map_left(AnyQueryResult::Sqlite)
                            .map_right(AnyRow::Sqlite)
                    })
                }))
            }
        }
    }
}

/// [`Future`] returned by [`AnyQuery::fetch_optional`]
pub enum AnyFetchOptional<'exe> {
    #[cfg(feature = "postgres")]
    Postgres {
        future: BoxFuture<'exe, sqlx::Result<Option<postgres::PgRow>>>,
        span: Span,
    },
    #[cfg(feature = "sqlite")]
    Sqlite {
        future: BoxFuture<'exe, sqlx::Result<Option<sqlite::SqliteRow>>>,
        span: Span,
    },
}
impl Unpin for AnyFetchOptional<'_> {}
impl Future for AnyFetchOptional<'_> {
    type Output = sqlx::Result<Option<AnyRow>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this {
            #[cfg(feature = "postgres")]
            Self::Postgres { future, span } => {
                let res = ready!(span.in_scope(|| future.as_mut().poll(cx)));
                *span = Span::none(); // Drop span after first item
                Poll::Ready(res.map(|opt| opt.map(AnyRow::Postgres)))
            }
            #[cfg(feature = "sqlite")]
            Self::Sqlite { future, span } => {
                let res = ready!(span.in_scope(|| future.as_mut().poll(cx)));
                *span = Span::none(); // Drop span after first item
                Poll::Ready(res.map(|opt| opt.map(AnyRow::Sqlite)))
            }
        }
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
                span: QuerySpanBuilder::new(&query),
                executor: pool,
                query: Some(sqlx::query(query)),
            },
            #[cfg(feature = "sqlite")]
            AnyPool::Sqlite(pool) => AnyQuery::SqlitePool {
                span: QuerySpanBuilder::new(&query),
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
                span: QuerySpanBuilder::new(&query),
                executor: tx.deref_mut(),
                query: Some(sqlx::query(query)),
            },
            #[cfg(feature = "sqlite")]
            AnyTransaction::Sqlite(tx) => AnyQuery::SqliteConn {
                span: QuerySpanBuilder::new(&query),
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
