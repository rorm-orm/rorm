//! [`Database`] struct and several common operations

use std::sync::Arc;

use rorm_declaration::config::DatabaseDriver;
use rorm_sql::conditional;
use rorm_sql::delete::Delete;
use rorm_sql::insert::Insert;
use rorm_sql::join_table::JoinTableData;
use rorm_sql::ordering::OrderByEntry;
use rorm_sql::select::Select;
use rorm_sql::select_column::SelectColumnData;
use rorm_sql::update::Update;
use rorm_sql::value::Value;
use tracing::warn;

use crate::error::Error;
use crate::executor::{AffectedRows, All, Executor, Nothing, One, QueryStrategy};
use crate::internal::any::AnyPool;
use crate::query_type::GetLimitClause;
use crate::row::Row;
use crate::transaction::Transaction;

/**
Type alias for [`SelectColumnData`]..

As all databases use currently the same fields, a type alias is sufficient.
*/
pub type ColumnSelector<'a> = SelectColumnData<'a>;

/**
Type alias for [`JoinTableData`].

As all databases use currently the same fields, a type alias is sufficient.
*/
pub type JoinTable<'until_build, 'post_build> = JoinTableData<'until_build, 'post_build>;

/// Configuration use in [`Database::connect`].
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DatabaseConfiguration {
    /// The driver and its corresponding settings
    pub driver: DatabaseDriver,

    /// Minimal connections to initialize upfront.
    ///
    /// Must be greater than `0` and can't be larger than `max_connections`.
    pub min_connections: u32,

    /// Maximum connections that allowed to be created.
    ///
    /// Must be greater than `0`.
    pub max_connections: u32,
}

impl DatabaseConfiguration {
    /**
    Create a new database configuration with some defaults set.

    **Defaults**:
    - `min_connections`: 1
    - `max_connections`: 10

    **Parameter**:
    - `driver`: [`DatabaseDriver`]: Configuration of the database driver.
    */
    pub fn new(driver: DatabaseDriver) -> Self {
        DatabaseConfiguration {
            driver,
            min_connections: 1,
            max_connections: 10,
        }
    }
}

/// Handle to a pool of database connections
///
/// Executing sql statements is done through the [`Executor`] trait
/// which is implemented on `&Database`.
///
/// Common operations are implemented as functions in the [`database`](self) module.
///
/// Cloning is cheap i.e. two `Arc`s.
#[derive(Clone)]
pub struct Database(pub(crate) AnyPool, Arc<()>);

impl Database {
    /// Connects to the database using `configuration`
    pub async fn connect(configuration: DatabaseConfiguration) -> Result<Self, Error> {
        Ok(Self(AnyPool::connect(configuration).await?, Arc::new(())))
    }

    /// Starts a new transaction
    ///
    /// `&mut Transaction` implements [`Executor`] like `&Database` does
    /// but its database operations can be reverted using [`Transaction::rollback`]
    /// or simply dropping the transaction without calling [`Transaction::commit`].
    pub async fn start_transaction(&self) -> Result<Transaction, Error> {
        Ok(Transaction::new(self.0.begin().await?))
    }

    /// Closes the database connection
    ///
    /// While calling this method is not strictly necessary,
    /// terminating your program without it
    /// might result in some final queries not being flushed properly.
    ///
    /// This method consumes the database handle,
    /// but actually all handles created using `clone` will become invalid after this call.
    /// This means any further operation would result in an `Err`
    pub async fn close(self) {
        self.0.close().await;
    }
}

impl Drop for Database {
    /// Checks whether [`Database::close`] has been called before the last instance is dropped
    fn drop(&mut self) {
        // The use of strong_count should be correct:
        // - the arc is private and we don't create WeakRefs
        // => when observing a strong_count of 1, there can't be any remaining refs
        if Arc::strong_count(&self.1) == 1 && !self.0.is_closed() {
            warn!("Database has been dropped without calling close. This might case the last queries to not being flushed properly");
        }
    }
}

/// Executes a simple `SELECT` query.
///
/// It is generic over a [`QueryStrategy`] which specifies how and how many rows to query.
///
/// **Parameter**:
/// - `model`: Model to query.
/// - `columns`: Columns to retrieve values from.
/// - `joins`: Join tables expressions.
/// - `conditions`: Optional conditions to apply.
/// - `order_by_clause`: Columns to order the rows by.
/// - `limit`: Optional limit / offset to apply to the query.
///   Depending on the query strategy, this is either [`LimitClause`](rorm_sql::limit_clause::LimitClause)
///   (for [`All`] and [`Stream`](crate::executor::Stream))
///   or a simple [`u64`] (for [`One`] and [`Optional`](crate::executor::Optional)).
#[allow(clippy::too_many_arguments)]
pub fn query<'result, 'db: 'result, 'post_query: 'result, Q: QueryStrategy + GetLimitClause>(
    executor: impl Executor<'db>,
    model: &str,
    columns: &[ColumnSelector<'_>],
    joins: &[JoinTable<'_, 'post_query>],
    conditions: Option<&conditional::Condition<'post_query>>,
    order_by_clause: &[OrderByEntry<'_>],
    limit: Option<Q::LimitOrOffset>,
) -> Q::Result<'result> {
    let columns: Vec<_> = columns
        .iter()
        .map(|c| {
            executor.dialect().select_column(
                c.table_name,
                c.column_name,
                c.select_alias,
                c.aggregation,
            )
        })
        .collect();
    let joins: Vec<_> = joins
        .iter()
        .map(|j| {
            executor.dialect().join_table(
                j.join_type,
                j.table_name,
                j.join_alias,
                j.join_condition.clone(),
            )
        })
        .collect();
    let mut q = executor
        .dialect()
        .select(&columns, model, &joins, order_by_clause);

    if let Some(condition) = conditions {
        q = q.where_clause(condition);
    }

    if let Some(limit) = Q::get_limit_clause(limit) {
        q = q.limit_clause(limit);
    }

    let (query_string, bind_params) = q.build();

    executor.execute::<Q>(query_string, bind_params)
}

/// Inserts a single row and returns columns from it.
///
/// **Parameter**:
/// - `model`: Table to insert to
/// - `columns`: Columns to set `values` for.
/// - `values`: Values to bind to the corresponding columns.
/// - `returning`: Columns to query from the inserted row.
pub async fn insert_returning(
    executor: impl Executor<'_>,
    model: &str,
    columns: &[&str],
    values: &[Value<'_>],
    returning: &[&str],
) -> Result<Row, Error> {
    generic_insert::<One>(executor, model, columns, values, Some(returning)).await
}

/// Inserts a single row.
///
/// **Parameter**:
/// - `model`: Table to insert to
/// - `columns`: Columns to set `values` for.
/// - `values`: Values to bind to the corresponding columns.
pub async fn insert(
    executor: impl Executor<'_>,
    model: &str,
    columns: &[&str],
    values: &[Value<'_>],
) -> Result<(), Error> {
    generic_insert::<Nothing>(executor, model, columns, values, None).await
}

/// Generic implementation of:
/// - [`Database::insert`]
/// - [`Database::insert_returning`]
pub(crate) fn generic_insert<'result, 'db: 'result, 'post_query: 'result, Q: QueryStrategy>(
    executor: impl Executor<'db>,
    model: &str,
    columns: &[&str],
    values: &[Value<'post_query>],
    returning: Option<&[&str]>,
) -> Q::Result<'result> {
    let values = &[values];
    let q = executor.dialect().insert(model, columns, values, returning);

    let (query_string, bind_params): (_, Vec<Value<'post_query>>) = q.build();

    executor.execute::<Q>(query_string, bind_params)
}

/// This method is used to bulk insert rows.
///
/// If one insert statement fails, the complete operation will be rolled back.
///
/// **Parameter**:
/// - `model`: Table to insert to
/// - `columns`: Columns to set `rows` for.
/// - `rows`: List of values to bind to the corresponding columns.
/// - `transaction`: Optional transaction to execute the query on.
pub async fn insert_bulk(
    executor: impl Executor<'_>,
    model: &str,
    columns: &[&str],
    rows: &[&[Value<'_>]],
) -> Result<(), Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tr: &mut Transaction = guard.get_transaction();

    for chunk in rows.chunks(25) {
        let mut insert = tr.dialect().insert(model, columns, chunk, None);
        insert = insert.rollback_transaction();
        let (insert_query, insert_params) = insert.build();

        tr.execute::<Nothing>(insert_query, insert_params).await?;
    }

    guard.commit().await?;
    Ok(())
}

/// This method is used to bulk insert rows.
///
/// If one insert statement fails, the complete operation will be rolled back.
///
/// **Parameter**:
/// - `model`: Table to insert to
/// - `columns`: Columns to set `rows` for.
/// - `rows`: List of values to bind to the corresponding columns.
/// - `transaction`: Optional transaction to execute the query on.
pub async fn insert_bulk_returning(
    executor: impl Executor<'_>,
    model: &str,
    columns: &[&str],
    rows: &[&[Value<'_>]],
    returning: &[&str],
) -> Result<Vec<Row>, Error> {
    let mut guard = executor.ensure_transaction().await?;
    let tr: &mut Transaction = guard.get_transaction();

    let mut inserted = Vec::with_capacity(rows.len());
    for chunk in rows.chunks(25) {
        let mut insert = tr.dialect().insert(model, columns, chunk, Some(returning));
        insert = insert.rollback_transaction();
        let (insert_query, insert_params) = insert.build();

        inserted.extend(tr.execute::<All>(insert_query, insert_params).await?);
    }

    guard.commit().await?;

    Ok(inserted)
}

/// This method is used to delete rows from a table.
///
/// **Parameter**:
/// - `model`: Name of the model to delete rows from
/// - `condition`: Optional condition to apply.
/// - `transaction`: Optional transaction to execute the query on.
///
/// **Returns** the rows affected of the delete statement. Note that this also includes
/// relations, etc.
pub async fn delete<'post_build>(
    executor: impl Executor<'_>,
    model: &str,
    condition: Option<&conditional::Condition<'post_build>>,
) -> Result<u64, Error> {
    let mut q = executor.dialect().delete(model);
    if let Some(condition) = condition {
        q = q.where_clause(condition);
    }

    let (query_string, bind_params) = q.build();

    executor
        .execute::<AffectedRows>(query_string, bind_params)
        .await
}

/// This method is used to update rows in a table.
///
/// **Parameter**:
/// - `model`: Name of the model to update rows from
/// - `updates`: A list of updates. An update is a tuple that consists of a list of columns to
///   update as well as the value to set to the columns.
/// - `condition`: Optional condition to apply.
/// - `transaction`: Optional transaction to execute the query on.
///
/// **Returns** the rows affected from the update statement. Note that this also includes
/// relations, etc.
pub async fn update<'post_build>(
    executor: impl Executor<'_>,
    model: &str,
    updates: &[(&str, Value<'post_build>)],
    condition: Option<&conditional::Condition<'post_build>>,
) -> Result<u64, Error> {
    let mut stmt = executor.dialect().update(model);

    for (column, value) in updates {
        stmt = stmt.add_update(column, *value);
    }

    if let Some(cond) = condition {
        stmt = stmt.where_clause(cond);
    }

    let (query_string, bind_params) = stmt.build()?;

    executor
        .execute::<AffectedRows>(query_string, bind_params)
        .await
}
