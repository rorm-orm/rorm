//! Delete builder and macro

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use rorm_db::error::Error;
use rorm_db::transaction::Transaction;
use rorm_db::Database;

use crate::conditions::{Condition, DynamicCollection};
use crate::crud::builder::{ConditionMarker, TransactionMarker};
use crate::internal::query_context::QueryContext;
use crate::model::{Identifiable, Model, PatchAsCondition};
use crate::Patch;

/// Builder for delete queries
///
/// Is is recommended to start a builder using [delete!](macro@crate::delete).
///
/// ## Generics
/// - `'rf`
///
///     Lifetime of external values (eg: condition values and transaction reference).
///
/// - `'db: 'rf`
///
///     The database reference's lifetime.
///     Since `'rf` also applies to a transaction reference, `'db` must outlive `'rf`.
///
/// - `M`: [Model](Model)
///
///     The model from whose table to delete rows.
///
/// - `C`: [ConditionMarker<'rf>](ConditionMarker)
///
///     An optional condition to filter the query by.
///
/// - `T`: [TransactionMarker<'rf,' db>](TransactionMarker)
///
///     An optional transaction to execute this query in.
///
#[must_use]
pub struct DeleteBuilder<'db, 'rf, M, C, T> {
    db: &'db Database,
    condition: C,
    transaction: T,

    _phantom: PhantomData<&'rf M>,
}

impl<'db, 'rf, M> DeleteBuilder<'db, 'rf, M, (), ()>
where
    M: Model,
{
    /// Start building a delete query
    pub fn new(db: &'db Database) -> Self {
        DeleteBuilder {
            db,
            condition: (),
            transaction: (),

            _phantom: PhantomData,
        }
    }
}

impl<'db, 'rf, M, T> DeleteBuilder<'db, 'rf, M, (), T>
where
    M: Model,
{
    /// Set the condition to delete a single model instance
    pub fn single<P>(self, patch: &'rf P) -> DeleteBuilder<'db, 'rf, M, PatchAsCondition<'rf, M>, T>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(patch.as_condition())
    }

    /// Set the condition to delete a bulk of model instances
    pub fn bulk<P>(
        self,
        patches: impl IntoIterator<Item = &'rf P>,
    ) -> DeleteBuilder<'db, 'rf, M, DynamicCollection<PatchAsCondition<'rf, M>>, T>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(DynamicCollection::or(
            patches
                .into_iter()
                .map(|patch| patch.as_condition())
                .collect(),
        ))
    }

    /// Add a condition to the delete query
    pub fn condition<C: Condition<'rf>>(self, condition: C) -> DeleteBuilder<'db, 'rf, M, C, T> {
        #[rustfmt::skip]
        let DeleteBuilder { db, transaction, _phantom, .. } = self;
        #[rustfmt::skip]
        return DeleteBuilder { db, condition, transaction, _phantom, };
    }
}

impl<'db, 'rf, M, C> DeleteBuilder<'db, 'rf, M, C, ()> {
    /// Add a transaction to the delete query
    pub fn transaction(
        self,
        transaction: &'rf mut Transaction<'db>,
    ) -> DeleteBuilder<'db, 'rf, M, C, &'rf mut Transaction<'db>> {
        #[rustfmt::skip]
        let DeleteBuilder { db, condition, _phantom, .. } = self;
        #[rustfmt::skip]
        return DeleteBuilder { db, condition, transaction, _phantom, };
    }
}

impl<'db, 'rf, M, T> DeleteBuilder<'db, 'rf, M, (), T>
where
    'db: 'rf,
    M: Model,
    T: TransactionMarker<'rf, 'db>,
{
    /// Delete all columns
    pub async fn all(self) -> Result<u64, Error> {
        self.db
            .delete(M::TABLE, None, self.transaction.into_option())
            .await
    }
}

impl<'db, 'rf, M, C, T> DeleteBuilder<'db, 'rf, M, C, T>
where
    'db: 'rf,
    M: Model,
    C: ConditionMarker<'rf>,
    T: TransactionMarker<'rf, 'db>,
{
    /// Perform the delete operation
    async fn exec(self) -> Result<u64, Error> {
        let context = QueryContext::new();
        self.db
            .delete(
                M::TABLE,
                self.condition.into_option(&context).as_ref(),
                self.transaction.into_option(),
            )
            .await
    }
}

impl<'db, 'rf, M, C, T> IntoFuture for DeleteBuilder<'db, 'rf, M, C, T>
where
    'db: 'rf,
    M: Model,
    C: Condition<'rf>,
    T: TransactionMarker<'rf, 'db>,
{
    type Output = Result<u64, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'rf>>;

    /// Convert a [DeleteBuilder] with a [Condition] into a [Future] implicitly
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

/// Create a DELETE query.
///
/// 1. Give a reference to your db and the model whose table's rows you want to delete
///
///     `delete!(&db, MyModelType)`
///
/// 2. *Optionally* add this query to a transaction
///
///     `.transaction(&mut tr)`
///
/// 3. Specify what to delete. This can be done using an arbitrary [condition], a [single] instance or a [bulk] of instances.
///
///     `.condition(MyModelType::F.id.greater(0))`
///
///     `.single(&my_instance)`
///
///     `.bulk(&[one_instances, another_instance])`
///
/// 4. Execute. After step 3 you could already `.await`ed your query.
///
///     If you want to skip step 3, you'd have to call [`all`] to make sure you want a DELETE query without any condition.
///
/// Example:
/// ```no_run
/// # use rorm::{Model, Database, delete};
/// #
/// # #[derive(Model)]
/// # struct User {
/// #     #[rorm(id)]
/// #     id: i64,
/// #
/// #     age: i32,
/// # }
/// #
/// pub async fn delete_underaged(db: &Database) {
///     delete!(db, User)
///         .condition(User::F.age.less(18))
///         .await
///         .unwrap();
/// }
/// ```
///
/// [condition]: DeleteBuilder::condition
/// [single]: DeleteBuilder::single
/// [bulk]: DeleteBuilder::bulk
/// [`all`]: DeleteBuilder::all
#[macro_export]
macro_rules! delete {
    ($db:expr, $model:path) => {
        $crate::crud::delete::DeleteBuilder::<$model, _, _>::new($db)
    };
}
