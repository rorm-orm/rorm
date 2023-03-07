//! Update builder and macro

use std::future::IntoFuture;
use std::marker::PhantomData;

use futures::future::BoxFuture;
use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::{Condition, IntoSingleValue, Value};
use crate::crud::builder::ConditionMarker;
use crate::internal::field::{Field, FieldProxy};
use crate::internal::query_context::QueryContext;
use crate::Model;

/// Wrapper around `Vec` to indicate on type level, that possible no column has been set yet.
pub struct OptionalColumns<'a>(Vec<(&'static str, Value<'a>)>);

/// Builder for update queries
///
/// Is is recommended to start a builder using [`update!`](macro@crate::update).
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
/// - `M`: [`Model`](Model)
///
///     The model from whose table to update rows.
///
/// - `L`
///
///     List of columns and values to set.
///     This is a generic instead of just being a `Vec` in order to prevent the list from being empty.
///
/// - `C`: [`ConditionMarker<'rf>`](ConditionMarker)
///
///     An optional condition to filter the query by.
///
#[must_use]
pub struct UpdateBuilder<'rf, E, M, L, C> {
    executor: E,
    columns: L,
    condition: C,

    _phantom: PhantomData<&'rf M>,
}

impl<'rf, 'e, E, M> UpdateBuilder<'rf, E, M, (), ()>
where
    E: Executor<'e>,
    M: Model,
{
    /// Start building a delete query
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            columns: (),
            condition: (),

            _phantom: PhantomData,
        }
    }
}

impl<'rf, E, M, L> UpdateBuilder<'rf, E, M, L, ()> {
    /// Add a condition to the query
    pub fn condition<C: Condition<'rf>>(self, condition: C) -> UpdateBuilder<'rf, E, M, L, C> {
        #[rustfmt::skip]
        let UpdateBuilder { executor, columns, _phantom, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { executor, columns, _phantom, condition, };
    }
}

impl<'rf, E, M, C> UpdateBuilder<'rf, E, M, (), C> {
    /// Prepare the builder to accept a dynamic (possibly zero) amount of set calls.
    ///
    /// Call [`finish_dyn_set`](UpdateBuilder::finish_dyn_set) to go back to normal operation.
    ///
    /// Normally `set` would use the type system to ensure that it has been called at least once
    /// before executing the query.
    /// This can be troublesome, when you want to call it dynamically
    /// and can't ensure that at least one such call will happen.
    pub fn begin_dyn_set(self) -> UpdateBuilder<'rf, E, M, OptionalColumns<'rf>, C> {
        #[rustfmt::skip]
        let UpdateBuilder { executor, _phantom, condition, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { executor, columns: OptionalColumns(Vec::new()), _phantom, condition, };
    }
}

impl<'rf, E, M, C> UpdateBuilder<'rf, E, M, OptionalColumns<'rf>, C> {
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<F: Field>(
        self,
        _field: FieldProxy<F, M>,
        value: impl IntoSingleValue<'rf, <F as Field>::DbType, Condition = Value<'rf>>,
    ) -> Self {
        let mut builder = self;
        builder.columns.0.push((F::NAME, value.into_condition()));
        builder
    }

    /// Go back to a "normal" builder after calling [`begin_dyn_set`](UpdateBuilder::begin_dyn_set).
    ///
    /// This will check if `set` has been called at least once.
    /// If it hasn't, the "unset" builder will be returned as `Err`.
    pub fn finish_dyn_set(
        self,
    ) -> Result<UpdateBuilderWithSet<'rf, E, M, C>, UpdateBuilderWithoutSet<'rf, E, M, C>> {
        #[rustfmt::skip]
        let UpdateBuilder { executor, _phantom, condition, columns } = self;
        #[rustfmt::skip]
        return if columns.0.is_empty() {
            Err(UpdateBuilder { executor, columns: (), _phantom, condition, })
        } else {
            Ok(UpdateBuilder { executor, columns: columns.0, _phantom, condition, })
        };
    }
}
type UpdateBuilderWithoutSet<'rf, E, M, C> = UpdateBuilder<'rf, E, M, (), C>;
type UpdateBuilderWithSet<'rf, E, M, C> =
    UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C>;

impl<'rf, E, M, C> UpdateBuilder<'rf, E, M, (), C>
where
    M: Model,
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<F: Field>(
        self,
        _field: FieldProxy<F, M>,
        value: impl IntoSingleValue<'rf, <F as Field>::DbType, Condition = Value<'rf>>,
    ) -> UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C> {
        #[rustfmt::skip]
        let UpdateBuilder { executor, _phantom, condition, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { executor, columns: vec![(F::NAME, value.into_condition())], _phantom, condition, };
    }
}

impl<'rf, E, M, C> UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C>
where
    M: Model,
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<F: Field>(
        self,
        _field: FieldProxy<F, M>,
        value: impl IntoSingleValue<'rf, <F as Field>::DbType, Condition = Value<'rf>>,
    ) -> Self {
        let mut builder = self;
        builder.columns.push((F::NAME, value.into_condition()));
        builder
    }
}

impl<'ex, 'rf, E, M, C> UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C>
where
    E: Executor<'ex> + 'ex,
    M: Model,
    C: ConditionMarker<'rf>,
{
    /// Perform the update operation
    pub async fn exec<'fut>(self) -> Result<u64, Error>
    where
        'ex: 'fut,
        'rf: 'fut,
    {
        let context = QueryContext::new();
        let columns: Vec<_> = self
            .columns
            .iter()
            .map(|(name, value)| (*name, value.as_sql()))
            .collect();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&context));

        database::update(self.executor, M::TABLE, &columns, condition.as_ref()).await
    }
}

impl<'rf, E, M, C> IntoFuture for UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C>
where
    E: Executor<'rf> + 'rf + Send,
    M: Model + Sync,
    C: ConditionMarker<'rf>,
{
    type Output = Result<u64, Error>;
    type IntoFuture = BoxFuture<'rf, Self::Output>;

    /// Convert a [`UpdateBuilder`] with columns into a future implicitly
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

/// Create a UPDATE query.
///
/// # Basic usage
/// ```no_run
/// # use rorm::{Model, Database, update};
/// # #[derive(Model)] struct User { #[rorm(id)] id: i64, #[rorm(max_length = 255)] password: String, }
/// pub async fn set_good_password(db: &Database) {
///     update!(db, User)
///         .set(User::F.password, "I am way more secure™")
///         .condition(User::F.password.equals("password"))
///         .await
///         .unwrap();
/// }
/// ```
///
/// Like every crud macro `update!` starts a [builder](UpdateBuilder) which is consumed to execute the query.
///
/// `update!`'s first argument is a reference to the [`Database`](crate::Database).
/// Its second is the [`Model`] type you want to update rows of.
///
/// # Dynamic number of [`set`](UpdateBuilder::set)
/// ```no_run
/// # use std::collections::HashMap;
/// # use rorm::{Model, Database, update};
/// # #[derive(Model)] struct User { #[rorm(id)] id: i64, #[rorm(max_length = 255)] nickname: String, #[rorm(max_length = 255)] password: String, }
/// /// POST endpoint allowing a user to change its nickname or password
/// pub async fn update_user(db: &Database, id: i64, post_params: HashMap<String, String>) {
///     let mut builder = update!(db, User).begin_dyn_set();
///
///     if let Some(password) = post_params.get("password") {
///         builder = builder.set(User::F.password, password);
///     }
///     if let Some(nickname) = post_params.get("nickname") {
///         builder = builder.set(User::F.nickname, nickname)
///     }
///
///     if let Ok(builder) = builder.finish_dyn_set() {
///         builder.condition(User::F.id.equals(id)).await.unwrap();
///     } else {
///         panic!("Invalid POST request: missing fields to update")
///     }
/// }
/// ```
///
/// Before executing the query [`set`](UpdateBuilder::set) has to be called at least once
/// to set a value to set for a column (The first call changes the builders type).
/// Otherwise the query wouldn't do anything.
///
/// This can be limiting when your calls are made conditionally.
///
/// To support this, the builder can be put into a "dynamic" mode by calling [begin_dyn_set](UpdateBuilder::begin_dyn_set).
/// Then calls to [`set`](UpdateBuilder::set) won't change the type.
/// When you're done use [finish_dyn_set](UpdateBuilder::finish_dyn_set) to go back to "normal" mode.
/// It will check the number of "sets" and return `Result` which is `Ok` for at least one and an
/// `Err` for zero.
/// Both variants contain the builder in "normal" mode to continue.
#[macro_export]
macro_rules! update {
    ($db:expr, $model:path) => {
        $crate::crud::update::UpdateBuilder::<_, $model, _, _>::new($db)
    };
}
