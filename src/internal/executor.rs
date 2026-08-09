use std::future;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use futures_core::stream;
use rorm_sql::value::Value;
use rorm_sql::DBImpl;
use sqlx::{AssertSqlSafe, SqlSafeStr, SqlStr};
use tracing::debug;

use crate::executor::{
    AffectedRows, All, DynamicExecutor, Executor, Nothing, One, Optional, QueryStrategy,
    QueryStrategyResult, Stream,
};
use crate::futures_util::{BoxFuture, BoxStream};
use crate::internal::any::{AnyExecutor, AnyPool, AnyQueryResult, AnyRow, AnyTransaction};
use crate::internal::bind_params::bind_param;
use crate::transaction::{Transaction, TransactionGuard};
use crate::{Database, Error, Row};

impl<'exe> Executor<'exe> for &'exe mut Transaction {
    fn execute<Q>(self, query: String, values: Vec<Value<'_>>) -> Q::Result<'exe>
    where
        Q: QueryStrategy,
    {
        debug!(
            target: "rorm_db::executor",
            sql = query,
            values.len = values.len(),
            "Executing statement"
        );
        Q::execute(&mut self.sqlx, AssertSqlSafe(query).into_sql_str(), values)
    }

    fn into_dyn(self) -> DynamicExecutor<'exe> {
        DynamicExecutor::Transaction(self)
    }

    fn dialect(&self) -> DBImpl {
        match self.sqlx {
            #[cfg(feature = "postgres")]
            AnyTransaction::Postgres(_) => DBImpl::Postgres,
            #[cfg(feature = "sqlite")]
            AnyTransaction::Sqlite(_) => DBImpl::SQLite,
        }
    }

    type EnsureTransactionFuture = Ready<Result<TransactionGuard<'exe>, Error>>;

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<TransactionGuard<'exe>, Error>> {
        Box::pin(ready(Ok(TransactionGuard::Borrowed(self))))
    }
}

impl<'exe> Executor<'exe> for &'exe Database {
    fn execute<Q>(self, query: String, values: Vec<Value<'_>>) -> Q::Result<'exe>
    where
        Q: QueryStrategy,
    {
        debug!(
            target: "rorm_db::executor",
            sql = query,
            values.len = values.len(),
            "Executing statement"
        );
        Q::execute(&self.0, AssertSqlSafe(query).into_sql_str(), values)
    }

    fn into_dyn(self) -> DynamicExecutor<'exe> {
        DynamicExecutor::Database(self)
    }

    fn dialect(&self) -> DBImpl {
        match self.0 {
            #[cfg(feature = "postgres")]
            AnyPool::Postgres(_) => DBImpl::Postgres,
            #[cfg(feature = "sqlite")]
            AnyPool::Sqlite(_) => DBImpl::SQLite,
        }
    }

    type EnsureTransactionFuture = BoxFuture<'exe, Result<TransactionGuard<'exe>, Error>>;

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<TransactionGuard<'exe>, Error>> {
        Box::pin(async move { self.start_transaction().await.map(TransactionGuard::Owned) })
    }
}

pub trait QueryStrategyImpl: QueryStrategyResult {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>;
}

impl QueryStrategyResult for Nothing {
    type Result<'query> = NothingFuture<'query>;
}

impl QueryStrategyImpl for Nothing {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        NothingFuture {
            stream: query.fetch_many(),
        }
    }
}

/// [`QueryStrategyResult::Result`] of [`Nothing`]
pub struct NothingFuture<'stream> {
    stream: BoxStream<'stream, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}

impl future::Future for NothingFuture<'_> {
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
    type Result<'query> = BoxFuture<'query, Result<u64, Error>>;
}

impl QueryStrategyImpl for AffectedRows {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        Box::pin(async move { Ok(query.fetch_affected_rows().await?) }) as BoxFuture<_>
    }
}

impl QueryStrategyResult for One {
    type Result<'query> = BoxFuture<'query, Result<Row, Error>>;
}

impl QueryStrategyImpl for One {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        Box::pin(async move {
            Ok(Row(query
                .fetch_optional()
                .await?
                .ok_or(sqlx::Error::RowNotFound)?))
        }) as BoxFuture<_>
    }
}

impl QueryStrategyResult for Optional {
    type Result<'query> = BoxFuture<'query, Result<Option<Row>, Error>>;
}

impl QueryStrategyImpl for Optional {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        Box::pin(async move { Ok(query.fetch_optional().await?.map(Row)) }) as BoxFuture<_>
    }
}

impl QueryStrategyResult for All {
    type Result<'query> = BoxFuture<'query, Result<Vec<Row>, Error>>;
}

impl QueryStrategyImpl for All {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        Box::pin(async move { Ok(query.fetch_all().await?.into_iter().map(Row).collect()) })
            as BoxFuture<_>
    }
}

impl QueryStrategyResult for Stream {
    type Result<'query> = StreamResult<'query>;
}

impl QueryStrategyImpl for Stream {
    fn execute<'exe, E>(executor: E, query: SqlStr, values: Vec<Value<'_>>) -> Self::Result<'exe>
    where
        E: AnyExecutor<'exe>,
    {
        let mut query = executor.query(query);
        for x in values {
            bind_param(&mut query, x);
        }
        StreamResult {
            stream: query.fetch_many(),
        }
    }
}

/// [`QueryStrategyResult::Result`] of [`Stream`]
pub struct StreamResult<'stream> {
    stream: BoxStream<'stream, sqlx::Result<sqlx::Either<AnyQueryResult, AnyRow>>>,
}

impl Unpin for StreamResult<'_> {}
impl stream::Stream for StreamResult<'_> {
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
