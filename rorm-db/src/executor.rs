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
pub trait Executor<'exe>: Sized {
    /// Executes a raw SQL query **without returning anything**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_nothing(self, query: String, values: Vec<Value<'_>>) -> FetchNothing<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns the number of rows affected**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_affected_rows(self, query: String, values: Vec<Value<'_>>) -> FetchAffectedRows<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns at most one row**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_optional(self, query: String, values: Vec<Value<'_>>) -> FetchOptional<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns exactly one row**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_one(self, query: String, values: Vec<Value<'_>>) -> FetchOne<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns all rows in a vec**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_all(self, query: String, values: Vec<Value<'_>>) -> FetchAll<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns all rows in a stream**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_stream(self, query: String, values: Vec<Value<'_>>) -> FetchStream<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Executes a raw SQL query **and returns TODO**
    ///
    /// The query is executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    fn fetch_raw(self, query: String, values: Vec<Value<'_>>) -> FetchRaw<'exe> {
        self.fetch_generic(query, values).into()
    }

    /// Prepares to execute a raw SQL query without specifying what the query should return
    ///
    /// The query will be executed as prepared statement.
    /// To bind parameter, use ? as placeholder in SQLite
    /// and $1, $2, $n in Postgres.
    ///
    /// This allows code to build the query
    /// and let the caller decide what should be returned (and how):
    ///
    /// ```rust,ignore
    /// fn complex_query(db: impl Executor<'exe>) -> FetchGeneric<'exe> {
    ///     db.fetch_generic(todo!("Construct a very"), todo!("complex query"))
    /// }
    ///
    /// // Caller chooses "all rows" as vec or stream
    /// let rows = complex_query(&db).all().await?;
    /// let stream = complex_query(&db).stream();
    /// ```
    ///
    /// Alternatively, all `Fetch...` types implement `From<FetchGeneric<'_>>`.
    /// This allows actual generic code:
    ///
    /// ```rust,ignore
    /// fn complex_query<Q: From<FetchGeneric<'exe>>(db: impl Executor<'exe>) -> Q {
    ///     db.fetch_generic(todo!("Construct a very"), todo!("complex query")).into()
    /// }
    ///
    /// // Caller chooses "all rows" as vec or stream
    /// let rows = complex_query::<FetchAll>(&db).await?;
    /// let stream = complex_query::<FetchStream>(&db);
    /// ```
    fn fetch_generic(self, query: String, values: Vec<Value<'_>>) -> FetchGeneric<'exe>;

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

/// A query which doesn't know yet what it should return.
///
/// Choose by calling a method or through an `Into` implementation.
///
/// Returned by [`Executor::fetch_generic`].
#[must_use = "Choose what the query should return"]
pub struct FetchGeneric<'exe> {
    pub(crate) inner: internal::any::AnyQuery<'exe>,
}

impl<'exe> FetchGeneric<'exe> {
    /// Executes the query **without returning anything**
    ///
    /// See [`Executor::fetch_nothing`]
    pub fn nothing(self) -> FetchNothing<'exe> {
        self.into()
    }

    /// Executes the query **and returns the number of rows affected**
    ///
    /// See [`Executor::fetch_affected_rows`]
    pub fn affected_rows(self) -> FetchAffectedRows<'exe> {
        self.into()
    }

    /// Executes the query **and returns at most one row**
    ///
    /// See [`Executor::fetch_optional`]
    pub fn optional(self) -> FetchOptional<'exe> {
        self.into()
    }

    /// Executes the query **and returns exactly one row**
    ///
    /// See [`Executor::fetch_one`]
    pub fn one(self) -> FetchOne<'exe> {
        self.into()
    }

    /// Executes the query **and returns all rows in a vec**
    ///
    /// See [`Executor::fetch_all`]
    pub fn all(self) -> FetchAll<'exe> {
        self.into()
    }

    /// Executes the query **and returns all rows in a stream**
    ///
    /// See [`Executor::fetch_stream`]
    pub fn stream(self) -> FetchStream<'exe> {
        self.into()
    }

    /// Executes the query **and returns TODO**
    ///
    /// See [`Executor::fetch_raw`]
    pub fn raw(self) -> FetchRaw<'exe> {
        self.into()
    }
}

/// `Future` returned by [`Executor::fetch_nothing`]
pub struct FetchNothing<'exe> {
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchNothing<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            stream: value.inner.fetch_many(),
        }
    }
}
impl Unpin for FetchNothing<'_> {}
impl Future for FetchNothing<'_> {
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

/// `Future` returned by [`Executor::fetch_affected_rows`]
pub struct FetchAffectedRows<'exe> {
    count: u64,
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchAffectedRows<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            count: 0,
            stream: value.inner.fetch_many(),
        }
    }
}
impl Unpin for FetchAffectedRows<'_> {}
impl Future for FetchAffectedRows<'_> {
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

/// `Future` returned by [`Executor::fetch_optional`]
pub struct FetchOptional<'exe> {
    future: BoxFuture<'exe, sqlx::Result<Option<AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchOptional<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            future: Box::pin(value.inner.fetch_optional()),
        }
    }
}
impl Unpin for FetchOptional<'_> {}
impl Future for FetchOptional<'_> {
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

/// `Future` returned by [`Executor::fetch_one`]
pub struct FetchOne<'exe> {
    future: BoxFuture<'exe, sqlx::Result<Option<AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchOne<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            future: Box::pin(value.inner.fetch_optional()),
        }
    }
}
impl Unpin for FetchOne<'_> {}
impl Future for FetchOne<'_> {
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

/// `Future` returned by [`Executor::fetch_all`]
pub struct FetchAll<'exe> {
    vec: Vec<Row>,
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchAll<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            vec: Vec::new(),
            stream: value.inner.fetch_many(),
        }
    }
}
impl Unpin for FetchAll<'_> {}
impl Future for FetchAll<'_> {
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

/// `Stream` returned by [`Executor::fetch_stream`]
pub struct FetchStream<'exe> {
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchStream<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            stream: value.inner.fetch_many(),
        }
    }
}
impl Unpin for FetchStream<'_> {}
impl StreamTrait for FetchStream<'_> {
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

/// `Stream` returned by [`Executor::fetch_raw`]
pub struct FetchRaw<'exe> {
    stream: BoxStream<'exe, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}
impl<'exe> From<FetchGeneric<'exe>> for FetchRaw<'exe> {
    fn from(value: FetchGeneric<'exe>) -> Self {
        Self {
            stream: value.inner.fetch_many(),
        }
    }
}
impl Unpin for FetchRaw<'_> {}
impl StreamTrait for FetchRaw<'_> {
    type Item = Result<!, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            return Poll::Ready(match ready!(self.stream.as_mut().poll_next(cx)) {
                None => None,
                Some(Err(error)) => Some(Err(error.into())),
                Some(Ok(_either)) => todo!(),
            });
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
    fn fetch_generic(self, query: String, values: Vec<Value<'_>>) -> FetchGeneric<'exe> {
        match self {
            DynamicExecutor::Database(db) => db.fetch_generic(query, values),
            DynamicExecutor::Transaction(tr) => tr.fetch_generic(query, values),
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
