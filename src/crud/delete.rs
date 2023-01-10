//! Delete builder and macro

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::{Condition, DynamicCollection};
use crate::crud::builder::ConditionMarker;
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
///     Lifetime of external values (eg: condition values).
///
/// - `E`: [`Executor`]
///
///     The executor to query with.
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
pub struct DeleteBuilder<'rf, E, M, C> {
    executor: E,
    condition: C,

    _phantom: PhantomData<&'rf M>,
}

impl<'ex, 'rf, E, M> DeleteBuilder<'rf, E, M, ()>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Start building a delete query
    pub fn new(executor: E) -> Self {
        DeleteBuilder {
            executor,
            condition: (),

            _phantom: PhantomData,
        }
    }
}

impl<'rf, E, M> DeleteBuilder<'rf, E, M, ()>
where
    M: Model,
{
    /// Set the condition to delete a single model instance
    pub fn single<P>(self, patch: &'rf P) -> DeleteBuilder<'rf, E, M, PatchAsCondition<'rf, M>>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(patch.as_condition())
    }

    /// Set the condition to delete a bulk of model instances
    pub fn bulk<P>(
        self,
        patches: impl IntoIterator<Item = &'rf P>,
    ) -> DeleteBuilder<'rf, E, M, DynamicCollection<PatchAsCondition<'rf, M>>>
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
    pub fn condition<C: Condition<'rf>>(self, condition: C) -> DeleteBuilder<'rf, E, M, C> {
        #[rustfmt::skip]
        let DeleteBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return DeleteBuilder { executor, condition, _phantom, };
    }
}

impl<'ex, 'rf, E, M> DeleteBuilder<'rf, E, M, ()>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Delete all columns
    pub async fn all(self) -> Result<u64, Error> {
        database::delete(self.executor, M::TABLE, None).await
    }
}

impl<'ex, 'rf, E, M, C> DeleteBuilder<'rf, E, M, C>
where
    E: Executor<'ex>,
    M: Model,
    C: ConditionMarker<'rf>,
{
    /// Perform the delete operation
    async fn exec(self) -> Result<u64, Error> {
        let context = QueryContext::new();
        database::delete(
            self.executor,
            M::TABLE,
            self.condition.into_option(&context).as_ref(),
        )
        .await
    }
}

impl<'ex, 'rf, E, M, C> IntoFuture for DeleteBuilder<'rf, E, M, C>
where
    'ex: 'rf,
    E: Executor<'ex> + 'ex,
    M: Model,
    C: Condition<'rf>,
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
        $crate::crud::delete::DeleteBuilder::<_, $model, _>::new($db)
    };
}
