//! This module defines a wrapper for sqlx's Executor
//!
//! Unlike sqlx's Executor which provides several separate methods for different querying strategies,
//! our [`Executor`] has a single method which is generic using the [`QueryStrategy`] trait.

use rorm_sql::value::Value;
use rorm_sql::DBImpl;
use tracing::debug;

use crate::futures_util::BoxFuture;
use crate::transaction::{Transaction, TransactionGuard};
use crate::{internal, Database, Error};

/// [`QueryStrategy`] returning nothing
///
/// `type Result<'result> = impl Future<Output = Result<(), Error>>`
pub struct Nothing;

impl QueryStrategy for Nothing {}

/// [`QueryStrategy`] returning how many rows have been affected by the query
///
/// `type Result<'result> = impl Future<Output = Result<u64, Error>>`
pub struct AffectedRows;

impl QueryStrategy for AffectedRows {}

/// [`QueryStrategy`] returning a single row
///
/// `type Result<'result> = impl Future<Output = Result<Row, Error>>`
pub struct One;

impl QueryStrategy for One {}

/// [`QueryStrategy`] returning an optional row
///
/// `type Result<'result> = impl Future<Output = Result<Option<Row>, Error>>`
pub struct Optional;

impl QueryStrategy for Optional {}

/// [`QueryStrategy`] returning a vector of rows
///
/// `type Result<'result> = impl Future<Output = Result<Vec<Row>, Error>>`
pub struct All;

impl QueryStrategy for All {}

/// [`QueryStrategy`] returning a stream of rows
///
/// `type Result<'result> = impl Stream<Item = Result<Row, Error>>`
pub struct Stream;

impl QueryStrategy for Stream {}

/// Define how a query is sent to and results retrieved from the database.
///
/// This trait is implemented on the following unit structs:
/// - [`Nothing`] retrieves nothing
/// - [`Optional`] retrieves an optional row
/// - [`One`] retrieves a single row
/// - [`Stream`] retrieves many rows in a stream
/// - [`All`] retrieves many rows in a vector
/// - [`AffectedRows`] returns the number of rows affected by the query
///
/// This trait has an associated `Result<'result>` type which is returned by [`Executor::execute`].
/// To avoid boxing, these types are quite big.
///
/// Each of those unit structs' docs (follow links above) contains an easy to read `impl Trait` version of the actual types.
pub trait QueryStrategy: QueryStrategyResult + internal::executor::QueryStrategyImpl {}

/// Helper trait to make the `Result<'exe>` public,
/// while keeping [`QueryStrategyImpl`](internal::executor::QueryStrategyImpl) itself private
#[doc(hidden)]
pub trait QueryStrategyResult {
    type Result<'exe>;
}

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
    /// To bind parameter, use ? as placeholder in SQLite and MySQL
    /// and $1, $2, $n in Postgres.
    ///
    /// The generic `Q` is used to "select" what the database is supposed to respond with.
    /// See [`QueryStrategy`] for a list of available options.
    ///
    /// ```skipped
    /// db.execute::<All>("SELECT * FROM foo;".to_string(), vec![]);
    /// ```
    fn execute<Q>(self, query: String, values: Vec<Value<'_>>) -> Q::Result<'exe>
    where
        Q: QueryStrategy;

    /// Get the executor's sql dialect.
    fn dialect(&self) -> DBImpl;

    /// Convenience method to convert into a "`dyn Executor`"
    fn into_dyn(self) -> DynamicExecutor<'exe>;

    /// Ensure a piece of code is run inside a transaction using a [`TransactionGuard`].
    ///
    /// In generic code an [`Executor`] might and might not be a `&mut Transaction`.
    /// But sometimes you'd want to ensure your code is run inside a transaction
    /// (for example [bulk inserts](crate::database::insert_bulk)).
    ///
    /// This method solves this by producing a type which is either an owned or borrowed Transaction
    /// depending on the [`Executor`] it is called on.
    fn ensure_transaction(self) -> BoxFuture<'exe, Result<TransactionGuard<'exe>, Error>>;
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
        match self {
            DynamicExecutor::Database(db) => db.execute::<Q>(query, values),
            DynamicExecutor::Transaction(tr) => tr.execute::<Q>(query, values),
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

    fn ensure_transaction(self) -> BoxFuture<'exe, Result<TransactionGuard<'exe>, Error>> {
        match self {
            DynamicExecutor::Database(db) => db.ensure_transaction(),
            DynamicExecutor::Transaction(tr) => Box::pin(tr.ensure_transaction()),
        }
    }
}
