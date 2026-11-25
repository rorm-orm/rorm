//! This module defines a wrapper for sqlx's Executor
//!
//! Unlike sqlx's Executor which provides several separate methods for different querying strategies,
//! our [`Executor`] has a single method which is generic using the [`QueryStrategy`] trait.

use futures_core::Stream as StreamTrait;
use rorm_sql::value::Value;
use rorm_sql::DBImpl;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use crate::futures_util::{BoxFuture, BoxStream};
use crate::internal::any::{AnyQueryResult, AnyRow};
pub use crate::query_type::All;
pub use crate::query_type::One;
pub use crate::query_type::Optional;
pub use crate::query_type::Stream;
use crate::transaction::{MaybeOwnedTransaction, Transaction};
use crate::{internal, Database, Error, Row};

/// Some kind of database connection which can execute queries
///
/// This trait is implemented by the database connection itself as well as transactions.
///
/// # Object Safety
/// This trait is **not** object safe.
/// However, there only exist two implementors,
/// which were combined into the [`DynamicExecutor`] enum.
pub trait Executor<'exe> {
    /// Executes a raw SQL query
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    ///
    /// The generic `Q` is used to "select" what the database is supposed to respond with.
    /// See [`QueryStrategy`] for a list of available options.
    ///
    /// ```skipped
    /// db.execute::<All>("SELECT * FROM foo;".to_string(), vec![]);
    /// ```
    fn execute(self, query: String, values: Vec<Value<'_>>) -> Query<'exe>;

    /// Get the executor's sql dialect.
    fn dialect(&self) -> DBImpl;

    /// Convenience method to convert into a "`dyn Executor`"
    fn into_dyn(self) -> DynamicExecutor<'exe>;

    /// Ensure a piece of code is run inside a transaction using a [`MaybeOwnedTransaction`].
    ///
    /// In generic code an [`Executor`] might and might not be a `&mut Transaction`.
    /// But sometimes you'd want to ensure your code is run inside a transaction
    /// (for example [bulk inserts](crate::database::insert_bulk)).
    ///
    /// This method solves this by producing a type which is either an owned or borrowed Transaction
    /// depending on the [`Executor`] it is called on.
    fn ensure_transaction(self) -> BoxFuture<'exe, Result<MaybeOwnedTransaction<'exe>, Error>>;
}

pub struct Query<'exe> {
    pub(crate) inner: internal::any::AnyQuery<'exe>,
}

impl<'exe> Query<'exe> {
    pub fn stream(self) -> QueryStream<'exe> {
        QueryStream {
            stream: self.inner.fetch_many(),
        }
    }

    pub fn nothing(self) -> QueryNothing<'exe> {
        QueryNothing {
            stream: self.inner.fetch_many(),
        }
    }

    pub fn all(self) -> QueryAll<'exe> {
        QueryAll {
            vec: Vec::new(),
            stream: self.inner.fetch_many(),
        }
    }

    pub fn optional(self) -> QueryOptional<'exe> {
        QueryOptional {
            future: Box::pin(self.inner.fetch_optional()),
        }
    }

    pub fn one(self) -> QueryOne<'exe> {
        QueryOne {
            future: Box::pin(self.inner.fetch_optional()),
        }
    }

    pub fn affected_rows(self) -> QueryAffected<'exe> {
        QueryAffected {
            count: 0,
            stream: self.inner.fetch_many(),
        }
    }
}

/// `Future` returned by [`Query::nothing`]
pub struct QueryNothing<'exe> {
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl Unpin for QueryNothing<'_> {}
impl Future for QueryNothing<'_> {
    type Output = Result<(), Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.stream.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Ready(Some(Err(x))) => return Poll::Ready(Err(x.into())),
                Poll::Ready(Some(Ok(_))) => continue,
            }
        }
    }
}

/// `Future` returned by [`Query::all`]
pub struct QueryAll<'exe> {
    vec: Vec<Row>,
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl Unpin for QueryAll<'_> {}
impl Future for QueryAll<'_> {
    type Output = Result<Vec<Row>, Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.stream.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(mem::take(&mut self.vec))),
                Poll::Ready(Some(Err(x))) => return Poll::Ready(Err(x.into())),
                Poll::Ready(Some(Ok(sqlx::Either::Left(_)))) => continue,
                Poll::Ready(Some(Ok(sqlx::Either::Right(row)))) => {
                    self.vec.push(Row(row));
                    continue;
                }
            }
        }
    }
}

/// `Stream` returned by [`Query::stream`]
pub struct QueryStream<'exe> {
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl Unpin for QueryStream<'_> {}
impl StreamTrait for QueryStream<'_> {
    type Item = Result<Row, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            return Poll::Ready(match ready!(self.stream.as_mut().poll_next(cx)) {
                None => None,
                Some(Err(error)) => Some(Err(error.into())),
                Some(Ok(sqlx::Either::Right(row))) => Some(Ok(Row(row))),
                Some(Ok(sqlx::Either::Left(_))) => continue,
            });
        }
    }
}

/// `Future` returned by [`Query::affected_rows`]
pub struct QueryAffected<'exe> {
    count: u64,
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl Unpin for QueryAffected<'_> {}
impl Future for QueryAffected<'_> {
    type Output = Result<u64, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.stream.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(self.count)),
                Poll::Ready(Some(Err(x))) => return Poll::Ready(Err(x.into())),
                Poll::Ready(Some(Ok(sqlx::Either::Left(result)))) => {
                    self.count = self.count.saturating_add(result.rows_affected());
                    continue;
                }
                Poll::Ready(Some(Ok(sqlx::Either::Right(_)))) => continue,
            }
        }
    }
}

/// `Future` returned by [`Query::optional`]
pub struct QueryOptional<'exe> {
    future: BoxFuture<'exe, sqlx::Result<Option<AnyRow>>>,
}
impl Unpin for QueryOptional<'_> {}
impl Future for QueryOptional<'_> {
    type Output = Result<Option<Row>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.future.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(x)) => Poll::Ready(Err(x.into())),
            Poll::Ready(Ok(None)) => Poll::Ready(Ok(None)),
            Poll::Ready(Ok(Some(x))) => Poll::Ready(Ok(Some(Row(x)))),
        }
    }
}

/// `Future` returned by [`Query::one`]
pub struct QueryOne<'exe> {
    future: BoxFuture<'exe, sqlx::Result<Option<AnyRow>>>,
}
impl Unpin for QueryOne<'_> {}
impl Future for QueryOne<'_> {
    type Output = Result<Row, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.future.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(x)) => Poll::Ready(Err(x.into())),
            Poll::Ready(Ok(None)) => Poll::Ready(Err(sqlx::Error::RowNotFound.into())),
            Poll::Ready(Ok(Some(x))) => Poll::Ready(Ok(Row(x))),
        }
    }
}

/// Choose whether to use transactions or not at runtime
///
/// Like a `Box<dyn Executor<'executor>>`
pub enum DynamicExecutor<'exe> {
    /// Use a default database connection
    Database(&'exe Database),
    /// Use a transaction
    Transaction(&'exe mut Transaction),
}

impl<'exe> Executor<'exe> for DynamicExecutor<'exe> {
    fn execute(self, query: String, values: Vec<Value<'_>>) -> Query<'exe> {
        match self {
            DynamicExecutor::Database(db) => db.execute(query, values),
            DynamicExecutor::Transaction(tr) => tr.execute(query, values),
        }
    }

    fn dialect(&self) -> DBImpl {
        match self {
            DynamicExecutor::Database(db) => db.dialect(),
            DynamicExecutor::Transaction(tr) => tr.dialect(),
        }
    }

    fn into_dyn(self) -> DynamicExecutor<'exe> {
        self
    }

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<MaybeOwnedTransaction<'exe>, Error>> {
        match self {
            DynamicExecutor::Database(db) => db.ensure_transaction(),
            DynamicExecutor::Transaction(tr) => Box::pin(tr.ensure_transaction()),
        }
    }
}
