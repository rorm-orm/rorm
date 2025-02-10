use std::future::Ready;

use rorm_sql::value::Value;
use rorm_sql::DBImpl;

use super::no_sqlx;
use crate::database::Database;
use crate::error::Error;
use crate::executor::{
    AffectedRows, All, DynamicExecutor, Executor, Nothing, One, Optional, QueryStrategy,
    QueryStrategyResult, Stream,
};
use crate::futures_util::{BoxFuture, EmptyStream};
use crate::row::Row;
use crate::transaction::{Transaction, TransactionGuard};

impl<'executor> Executor<'executor> for &'executor Database {
    fn execute<'data, 'result, Q>(
        self,
        _query: String,
        _values: Vec<Value<'data>>,
    ) -> Q::Result<'result>
    where
        'executor: 'result,
        'data: 'result,
        Q: QueryStrategy,
    {
        no_sqlx();
    }

    fn into_dyn(self) -> DynamicExecutor<'executor> {
        DynamicExecutor::Database(self)
    }

    fn dialect(&self) -> DBImpl {
        no_sqlx();
    }

    type EnsureTransactionFuture = Ready<Result<TransactionGuard<'executor>, Error>>;

    fn ensure_transaction(
        self,
    ) -> BoxFuture<'executor, Result<TransactionGuard<'executor>, Error>> {
        no_sqlx();
    }
}

impl<'executor> Executor<'executor> for &'executor mut Transaction {
    fn execute<'data, 'result, Q>(
        self,
        _query: String,
        _values: Vec<Value<'data>>,
    ) -> Q::Result<'result>
    where
        'executor: 'result,
        'data: 'result,
        Q: QueryStrategy,
    {
        no_sqlx();
    }

    fn into_dyn(self) -> DynamicExecutor<'executor> {
        DynamicExecutor::Transaction(self)
    }

    fn dialect(&self) -> DBImpl {
        no_sqlx();
    }

    type EnsureTransactionFuture = Ready<Result<TransactionGuard<'executor>, Error>>;

    fn ensure_transaction(
        self,
    ) -> BoxFuture<'executor, Result<TransactionGuard<'executor>, Error>> {
        no_sqlx();
    }
}

pub trait QueryStrategyImpl: QueryStrategyResult {}

impl<Q: QueryStrategyResult> QueryStrategyImpl for Q {}

impl QueryStrategyResult for Nothing {
    type Result<'result> = Ready<Result<(), Error>>;
}

impl QueryStrategyResult for AffectedRows {
    type Result<'result> = Ready<Result<u64, Error>>;
}

impl QueryStrategyResult for One {
    type Result<'result> = Ready<Result<Row, Error>>;
}

impl QueryStrategyResult for Optional {
    type Result<'result> = Ready<Result<Option<Row>, Error>>;
}

impl QueryStrategyResult for All {
    type Result<'result> = Ready<Result<Vec<Row>, Error>>;
}

impl QueryStrategyResult for Stream {
    type Result<'result> = EmptyStream<Result<Row, Error>>;
}
