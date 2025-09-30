//! Delete builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::{Condition, DynamicCollection};
use crate::crud::selector::Selector;
use crate::internal::patch::{IntoPatchCow, PatchCow};
use crate::internal::query_context::QueryContext;
use crate::model::{Identifiable, Model};
use crate::Patch;

/// Create a DELETE query.
///
/// # Usage
/// ```no_run
/// # use rorm::{Model, Patch, Database, delete};
/// # #[derive(Model)] pub struct User { #[rorm(id)] id: i64, age: i32, }
/// # #[derive(Patch)] #[rorm(model = "User")] pub struct UserPatch { id: i64, }
/// pub async fn delete_single_user(db: &Database, user: &UserPatch) {
///     delete(db, User)
///         .single(user)
///         .await
///         .unwrap();
/// }
/// pub async fn delete_many_users(db: &Database, users: &[UserPatch]) {
///     delete(db, User)
///         .bulk(users)
///         .await
///         .unwrap();
/// }
/// pub async fn delete_underage(db: &Database) {
///     let num_deleted: u64 = delete(db, User)
///         .condition(User.age.less_equals(18))
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
pub fn delete<'ex, E, S>(executor: E, _: S) -> DeleteBuilder<E, S::Model>
where
    E: Executor<'ex>,
    S: Selector<Model: Patch<ValueSpaceImpl = S>>,
{
    DeleteBuilder {
        executor,

        _phantom: PhantomData,
    }
}

/// Builder for delete queries
///
/// To create a builder use [`delete`]
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
    /// Delete a single row identified by a patch instance
    ///
    /// Note: The patch only provides the primary key, its other values will be ignored.
    pub async fn single<P>(self, patch: &P) -> Result<u64, Error>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(patch.as_condition()).await
    }

    /// Delete a bulk of rows identified by patch instances
    ///
    /// Note: The patches only provide the primary key, their other values will be ignored.
    ///
    /// # Argument
    /// This method accepts anything which can be used to iterate
    /// over instances or references of your [`Patch`].
    ///
    /// **Examples**: (where `P` is your patch)
    /// - `Vec<P>`
    /// - `&[P]`
    /// - A [`map`](Iterator::map) iterator yielding `P` or `&P`
    pub async fn bulk<'p, I, P>(self, patches: I) -> Result<u64, Error>
    where
        I: IntoIterator,
        I::Item: IntoPatchCow<'p, Patch = P>,
        P: Patch<Model = M> + Identifiable,
    {
        let mut owned = Vec::new();
        let mut conditions = Vec::new();
        for patch in patches {
            match patch.into_patch_cow() {
                PatchCow::Borrowed(patch) => conditions.push(patch.as_condition()),
                PatchCow::Owned(patch) => owned.push(patch),
            }
        }
        for patch in &owned {
            conditions.push(patch.as_condition());
        }
        if let Some(condition) = DynamicCollection::or_checked(conditions) {
            self.condition(condition).await
        } else {
            Ok(0)
        }
    }

    /// Delete all rows matching a condition
    pub async fn condition<'c, C: Condition<'c>>(self, condition: C) -> Result<u64, Error> {
        let mut context = QueryContext::new();
        let condition_index = context.add_condition(&condition);
        database::delete(
            self.executor,
            M::TABLE,
            Some(&context.get_condition(condition_index)),
        )
        .await
    }

    /// Delete all rows
    pub async fn all(self) -> Result<u64, Error> {
        database::delete(self.executor, M::TABLE, None).await
    }
}
