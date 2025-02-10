use std::future;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use futures_core::stream;
use rorm_sql::value::Value;
use rorm_sql::DBImpl;

use crate::executor::{
    AffectedRows, All, DynamicExecutor, Executor, Nothing, One, Optional, QueryStrategy,
    QueryStrategyResult, Stream,
};
use crate::internal::any::{AnyExecutor, AnyPool, AnyQueryResult, AnyRow, AnyTransaction};
use crate::transaction::{Transaction, TransactionGuard};
use crate::{Database, Error, Row};

impl<'executor> Executor<'executor> for &'executor mut Transaction {
    fn execute<'data, 'result, Q>(
        self,
        query: String,
        values: Vec<Value<'data>>,
    ) -> Q::Result<'result>
    where
        'executor: 'result,
        'data: 'result,
        Q: QueryStrategy,
    {
        Q::execute(&mut self.0, query, values)
    }

    fn into_dyn(self) -> DynamicExecutor<'executor> {
        DynamicExecutor::Transaction(self)
    }

    fn dialect(&self) -> DBImpl {
        match self.0 {
            #[cfg(feature = "postgres")]
            AnyTransaction::Postgres(_) => DBImpl::Postgres,
            #[cfg(feature = "mysql")]
            AnyTransaction::MySql(_) => DBImpl::MySQL,
            #[cfg(feature = "sqlite")]
            AnyTransaction::Sqlite(_) => DBImpl::SQLite,
        }
    }

    type EnsureTransactionFuture = Ready<Result<TransactionGuard<'executor>, Error>>;

    fn ensure_transaction(
        self,
    ) -> BoxFuture<'executor, Result<TransactionGuard<'executor>, Error>> {
        Box::pin(ready(Ok(TransactionGuard::Borrowed(self))))
    }
}

impl<'executor> Executor<'executor> for &'executor Database {
    fn execute<'data, 'result, Q>(
        self,
        query: String,
        values: Vec<Value<'data>>,
    ) -> Q::Result<'result>
    where
        'executor: 'result,
        'data: 'result,
        Q: QueryStrategy,
    {
        Q::execute(&self.0, query, values)
    }

    fn into_dyn(self) -> DynamicExecutor<'executor> {
        DynamicExecutor::Database(self)
    }

    fn dialect(&self) -> DBImpl {
        match self.0 {
            #[cfg(feature = "postgres")]
            AnyPool::Postgres(_) => DBImpl::Postgres,
            #[cfg(feature = "mysql")]
            AnyPool::MySql(_) => DBImpl::MySQL,
            #[cfg(feature = "sqlite")]
            AnyPool::Sqlite(_) => DBImpl::SQLite,
        }
    }

    type EnsureTransactionFuture = BoxFuture<'executor, Result<TransactionGuard<'executor>, Error>>;

    fn ensure_transaction(
        self,
    ) -> BoxFuture<'executor, Result<TransactionGuard<'executor>, Error>> {
        Box::pin(async move { self.start_transaction().await.map(TransactionGuard::Owned) })
    }
}

pub trait QueryStrategyImpl: QueryStrategyResult {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>;
}

pub type QueryFuture<T> = QueryWrapper<T>;
pub type QueryStream<T> = QueryWrapper<T>;

pub use query_wrapper::QueryWrapper;

use crate::futures_util::{BoxFuture, BoxStream};

/// Private module to contain the internals behind a sound api
mod query_wrapper {
    use std::pin::Pin;

    use rorm_sql::value::Value;

    use crate::internal::any::{AnyExecutor, AnyQuery};

    #[doc(hidden)]
    #[pin_project::pin_project]
    pub struct QueryWrapper<T> {
        #[pin]
        wrapped: T,
        #[allow(dead_code)] // is used via a reference inside T
        query_string: String,
    }

    impl<'query, T: 'query> QueryWrapper<T> {
        /// Basic constructor which only performs the unsafe lifetime extension to be tested by miri
        pub(crate) fn new_basic(string: String, wrapped: impl FnOnce(&'query str) -> T) -> Self {
            let slice: &str = string.as_str();

            // SAFETY: The heap allocation won't be dropped or moved
            //         until `wrapped` which contains this reference is dropped.
            let slice: &'query str = unsafe { std::mem::transmute(slice) };

            Self {
                query_string: string,
                wrapped: wrapped(slice),
            }
        }

        pub fn new<'data: 'query>(
            executor: impl AnyExecutor<'query>,
            query_string: String,
            values: Vec<Value<'data>>,
            execute: impl FnOnce(AnyQuery<'query>) -> T,
        ) -> Self {
            Self::new_basic(query_string, move |query_string| {
                let mut query = executor.query(query_string);
                for value in values {
                    crate::internal::utils::bind_param(&mut query, value);
                }
                execute(query)
            })
        }
    }

    impl<T> QueryWrapper<T> {
        /// Project a [`Pin`] onto the `wrapped` field
        pub fn project_wrapped(self: Pin<&mut Self>) -> Pin<&mut T> {
            self.project().wrapped
        }
    }
}

impl<F> future::Future for QueryFuture<F>
where
    F: future::Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project_wrapped().poll(cx)
    }
}

impl<S> stream::Stream for QueryStream<S>
where
    S: stream::Stream,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project_wrapped().poll_next(cx)
    }
}

impl QueryStrategyResult for Nothing {
    type Result<'query> = QueryFuture<NothingFuture<'query>>;
}

impl QueryStrategyImpl for Nothing {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryFuture::new(executor, query, values, |query| NothingFuture {
            stream: query.fetch_many(),
        })
    }
}

/// [`QueryStrategyResult::Result`] of [`Nothing`]
pub struct NothingFuture<'stream> {
    stream: BoxStream<'stream, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}

impl<'stream> future::Future for NothingFuture<'stream> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            return Poll::Ready(match ready!(self.stream.as_mut().poll_next(cx)) {
                None => Ok(()),
                Some(Err(error)) => Err(error.into()),
                Some(_either) => continue,
            });
        }
    }
}

impl QueryStrategyResult for AffectedRows {
    type Result<'query> = QueryFuture<BoxFuture<'query, Result<u64, Error>>>;
}

impl QueryStrategyImpl for AffectedRows {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryFuture::new(executor, query, values, |query| {
            Box::pin(async move { Ok(query.fetch_affected_rows().await?) }) as BoxFuture<_>
        })
    }
}

impl QueryStrategyResult for One {
    type Result<'query> = QueryFuture<BoxFuture<'query, Result<Row, Error>>>;
}

impl QueryStrategyImpl for One {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryFuture::new(executor, query, values, |query| {
            Box::pin(async move {
                Ok(Row(query
                    .fetch_optional()
                    .await?
                    .ok_or(sqlx::Error::RowNotFound)?))
            }) as BoxFuture<_>
        })
    }
}

impl QueryStrategyResult for Optional {
    type Result<'query> = QueryFuture<BoxFuture<'query, Result<Option<Row>, Error>>>;
}

impl QueryStrategyImpl for Optional {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryFuture::new(executor, query, values, |query| {
            Box::pin(async move { Ok(query.fetch_optional().await?.map(Row)) }) as BoxFuture<_>
        })
    }
}

impl QueryStrategyResult for All {
    type Result<'query> = QueryFuture<BoxFuture<'query, Result<Vec<Row>, Error>>>;
}

impl QueryStrategyImpl for All {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryFuture::new(executor, query, values, |query| {
            Box::pin(async move { Ok(query.fetch_all().await?.into_iter().map(Row).collect()) })
                as BoxFuture<_>
        })
    }
}

impl QueryStrategyResult for Stream {
    type Result<'query> = QueryStream<StreamResult<'query>>;
}

impl QueryStrategyImpl for Stream {
    fn execute<'query, E>(
        executor: E,
        query: String,
        values: Vec<Value<'query>>,
    ) -> Self::Result<'query>
    where
        E: AnyExecutor<'query>,
    {
        QueryStream::new(executor, query, values, |query| StreamResult {
            stream: query.fetch_many(),
        })
    }
}

/// [`QueryStrategyResult::Result`] of [`Stream`]
pub struct StreamResult<'stream> {
    stream: BoxStream<'stream, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}

impl<'stream> Unpin for StreamResult<'stream> {}
impl<'stream> stream::Stream for StreamResult<'stream> {
    type Item = Result<Row, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            return Poll::Ready(match ready!(self.stream.as_mut().poll_next(cx)) {
                None => None,
                Some(Err(error)) => Some(Err(error.into())),
                Some(Ok(sqlx::Either::Right(row))) => Some(Ok(Row(row))),
                Some(Ok(sqlx::Either::Left(_result))) => continue,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use crate::internal::executor::QueryWrapper;

    /// Run this test with miri
    ///
    /// If the drop order of [`QueryWrapper`]'s fields is incorrect,
    /// miri will complain about a use-after-free.
    #[test]
    fn test_drop_order() {
        struct BorrowStr<'a>(&'a str);
        impl<'a> Drop for BorrowStr<'a> {
            fn drop(&mut self) {
                // Use the borrowed string.
                // If it were already dropped, miri would detect it.
                println!("{}", self.0);
            }
        }
        let _w = QueryWrapper::new_basic(format!("Hello World"), BorrowStr);
    }
}
