//! Delete builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::{Condition, DynamicCollection};
use crate::internal::query_context::QueryContextBuilder;
use crate::model::{Identifiable, Model};
use crate::Patch;

/// Builder for delete queries
///
/// Is is recommended to start a builder using [`delete!`](macro@crate::delete).
///
/// ## Generics
/// - `E`: [`Executor`]
///
///     The executor to query with.
///
/// - `M`: [`Model`]
///
///     The model from whose table to delete rows.
///
#[must_use]
pub struct DeleteBuilder<E, M> {
    executor: E,

    _phantom: PhantomData<M>,
}

impl<'ex, E, M> DeleteBuilder<E, M>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Start building a delete query
    pub fn new(executor: E) -> Self {
        DeleteBuilder {
            executor,

            _phantom: PhantomData,
        }
    }
}

impl<'ex, E, M> DeleteBuilder<E, M>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Delete a single row identified by a patch instance
    pub async fn single<P>(self, patch: &P) -> Result<u64, Error>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(patch.as_condition()).await
    }

    /// Delete a bulk of rows identified by patch instances
    pub async fn bulk<P>(self, patches: impl IntoIterator<Item = &P>) -> Result<u64, Error>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(DynamicCollection::or(
            patches
                .into_iter()
                .map(|patch| patch.as_condition())
                .collect(),
        ))
        .await
    }

    /// Delete all rows matching a condition
    pub async fn condition<'c, C: Condition<'c>>(self, condition: C) -> Result<u64, Error> {
        let mut builder = QueryContextBuilder::new();
        condition.add_to_builder(&mut builder);
        let context = builder.finish();
        let condition = condition.as_sql(&context);
        database::delete(self.executor, M::TABLE, Some(&condition)).await
    }

    /// Delete all columns
    pub async fn all(self) -> Result<u64, Error> {
        database::delete(self.executor, M::TABLE, None).await
    }
}

/// Create a DELETE query.
///
/// # Usage
/// ```no_run
/// # use rorm::{Model, Patch, Database, delete};
/// # #[derive(Model)] pub struct User { #[rorm(id)] id: i64, age: i32, }
/// # #[derive(Patch)] #[rorm(model = "User")] pub struct UserPatch { id: i64, }
/// pub async fn delete_single_user(db: &Database, user: &UserPatch) {
///     delete!(db, User)
///         .single(user)
///         .await
///         .unwrap();
/// }
/// pub async fn delete_many_users(db: &Database, users: &[UserPatch]) {
///     delete!(db, User)
///         .bulk(users)
///         .await
///         .unwrap();
/// }
/// pub async fn delete_underage(db: &Database) {
///     let num_deleted: u64 = delete!(db, User)
///         .condition(User::F.age.less(18))
///         .await
///         .unwrap();
/// }
///```
///
/// Like every crud macro `delete!` starts a [builder](DeleteBuilder) which is consumed to execute the query.
///
/// `delete!`'s first argument is a reference to the [`Database`](crate::Database).
/// Its second is the [`Model`] type of whose table you want to delete columns from.
///
/// To specify what rows to delete use the following methods,
/// which will consume the builder and execute the query:
/// - [`single`](DeleteBuilder::single): Delete a single row identified by a patch instance
/// - [`bulk`](DeleteBuilder::bulk): Delete a bulk of rows identified by patch instances
/// - [`condition`](DeleteBuilder::condition): Delete all rows matching a condition
/// - [`all`](DeleteBuilder::all): Unconditionally delete all rows
#[macro_export]
macro_rules! delete {
    ($db:expr, $model:path) => {
        $crate::crud::delete::DeleteBuilder::<_, <$model as $crate::model::Patch>::Model>::new($db)
    };
}
