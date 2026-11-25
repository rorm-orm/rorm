use std::future::ready;

use rorm_sql::value::Value;
use rorm_sql::DBImpl;
use sqlx::{AssertSqlSafe, SqlSafeStr};

use crate::executor::{DynamicExecutor, Executor, Query};
use crate::futures_util::BoxFuture;
use crate::internal::any::{AnyExecutor, AnyPool};
use crate::internal::bind_params::bind_param;
use crate::transaction::{MaybeOwnedTransaction, Transaction};
use crate::{Database, Error};

impl<'exe> Executor<'exe> for &'exe mut Transaction {
    fn execute(self, query: String, values: Vec<Value<'_>>) -> Query<'exe> {
        let mut query = self.sqlx.query(AssertSqlSafe(query).into_sql_str());
        for value in values {
            bind_param(&mut query, value);
        }
        Query { inner: query }
    }

    fn into_dyn(self) -> DynamicExecutor<'exe> {
        DynamicExecutor::Transaction(self)
    }

    fn dialect(&self) -> DBImpl {
        Transaction::dialect(self)
    }

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<MaybeOwnedTransaction<'exe>, Error>> {
        Box::pin(ready(Ok(MaybeOwnedTransaction::Borrowed(self))))
    }
}

impl<'exe> Executor<'exe> for &'exe Database {
    fn execute(self, query: String, values: Vec<Value<'_>>) -> Query<'exe> {
        let mut query = self.0.query(AssertSqlSafe(query).into_sql_str());
        for value in values {
            bind_param(&mut query, value);
        }
        Query { inner: query }
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

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<MaybeOwnedTransaction<'exe>, Error>> {
        Box::pin(async move {
            self.start_transaction()
                .await
                .map(MaybeOwnedTransaction::Owned)
        })
    }
}
