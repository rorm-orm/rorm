//! Update builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::{Condition, DynamicCollection, Value};
use crate::crud::selector::Selector;
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::fields::utils::column_name::ColumnName;
use crate::internal::field::{Field, SingleColumnField};
use crate::internal::patch::{IntoPatchCow, PatchCow};
use crate::internal::query_context::QueryContext;
use crate::model::Identifiable;
use crate::{Model, Patch};

/// Create a UPDATE query.
///
/// # Basic usage
/// ```no_run
/// # use rorm::{Model, Database, update};
/// # #[derive(Model)] struct User { #[rorm(id)] id: i64, password: String, }
/// pub async fn set_good_password(db: &Database) {
///     update(db, User)
///         .set(User.password, "I am way more secure™".to_string())
///         .condition(User.password.equals("password"))
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
/// # #[derive(Model)] struct User { #[rorm(id)] id: i64, nickname: String, password: String, }
/// /// POST endpoint allowing a user to change its nickname or password
/// pub async fn update_user(db: &Database, id: i64, post_params: HashMap<String, String>) {
///     let mut builder = update(db, User).begin_dyn_set();
///
///     if let Some(password) = post_params.get("password") {
///         builder = builder.set(User.password, password.clone());
///     }
///     if let Some(nickname) = post_params.get("nickname") {
///         builder = builder.set(User.nickname, nickname.clone())
///     }
///
///     if let Ok(builder) = builder.finish_dyn_set() {
///         builder.condition(User.id.equals(id)).await.unwrap();
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
pub fn update<'rf, 'e, E, S>(executor: E, _: S) -> UpdateBuilder<'rf, E, S::Model, columns::Empty>
where
    E: Executor<'e>,
    S: Selector<Model: Patch<ValueSpaceImpl = S>>,
{
    UpdateBuilder {
        executor,
        columns: Vec::new(),
        _phantom: PhantomData,
    }
}

/// Builder for update queries
///
/// To create a builder use [`update`]
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
/// - `M`: [`Model`]
///
///     The model from whose table to update rows.
///
/// - `C`
///
///     Type state storing whether `set` has been called at least once.
#[must_use]
pub struct UpdateBuilder<'rf, E, M, C> {
    executor: E,
    columns: Vec<(&'static ColumnName, Value<'rf>)>,

    _phantom: PhantomData<(M, C)>,
}

/// Marker types representing [`UpdateBuilder`]'s state
#[doc(hidden)]
pub mod columns {
    pub struct Empty;
    pub struct NonEmpty;
    pub struct MaybeEmpty;
}

impl<'rf, E, M, C> UpdateBuilder<'rf, E, M, C> {
    fn set_column_state<C2>(self) -> UpdateBuilder<'rf, E, M, C2> {
        UpdateBuilder {
            executor: self.executor,
            columns: self.columns,
            _phantom: PhantomData,
        }
    }
}

impl<'rf, E, M> UpdateBuilder<'rf, E, M, columns::Empty> {
    /// Prepare the builder to accept a dynamic (possibly zero) amount of set calls.
    ///
    /// Call [`finish_dyn_set`](UpdateBuilder::finish_dyn_set) to go back to normal operation.
    ///
    /// Normally `set` would use the type system to ensure that it has been called at least once
    /// before executing the query.
    /// This can be troublesome, when you want to call it dynamically
    /// and can't ensure that at least one such call will happen.
    pub fn begin_dyn_set(self) -> UpdateBuilder<'rf, E, M, columns::MaybeEmpty> {
        self.set_column_state()
    }
}

impl<'rf, E, M> UpdateBuilder<'rf, E, M, columns::MaybeEmpty> {
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<I>(mut self, _field: FieldProxy<I>, value: <I::Field as Field>::Type) -> Self
    where
        I: FieldProxyImpl<Field: SingleColumnField, Path = M>,
    {
        self.columns.push((
            &<I::Field as Field>::NAME,
            <I::Field as SingleColumnField>::type_into_value(value),
        ));
        self
    }

    /// Add a column to update if `value` is `Some`
    ///
    /// Can be called multiple times.
    pub fn set_if<I>(self, field: FieldProxy<I>, value: Option<<I::Field as Field>::Type>) -> Self
    where
        I: FieldProxyImpl<Field: SingleColumnField, Path = M>,
    {
        if let Some(value) = value {
            self.set(field, value)
        } else {
            self
        }
    }

    /// Go back to a "normal" builder after calling [`begin_dyn_set`](UpdateBuilder::begin_dyn_set).
    ///
    /// This will check if `set` has been called at least once.
    /// If it hasn't, the "unset" builder will be returned as `Err`.
    pub fn finish_dyn_set(
        self,
    ) -> Result<UpdateBuilder<'rf, E, M, columns::NonEmpty>, UpdateBuilder<'rf, E, M, columns::Empty>>
    {
        if self.columns.is_empty() {
            Err(self.set_column_state())
        } else {
            Ok(self.set_column_state())
        }
    }
}

impl<'rf, E, M> UpdateBuilder<'rf, E, M, columns::Empty>
where
    M: Model,
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<I>(
        mut self,
        _field: FieldProxy<I>,
        value: <I::Field as Field>::Type,
    ) -> UpdateBuilder<'rf, E, M, columns::NonEmpty>
    where
        I: FieldProxyImpl<Field: SingleColumnField, Path = M>,
    {
        self.columns.push((
            &<I::Field as Field>::NAME,
            <I::Field as SingleColumnField>::type_into_value(value),
        ));
        self.set_column_state()
    }
}

impl<E, M> UpdateBuilder<'_, E, M, columns::NonEmpty>
where
    M: Model,
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<I>(mut self, _field: FieldProxy<I>, value: <I::Field as Field>::Type) -> Self
    where
        I: FieldProxyImpl<Field: SingleColumnField, Path = M>,
    {
        self.columns.push((
            &<I::Field as Field>::NAME,
            <I::Field as SingleColumnField>::type_into_value(value),
        ));
        self
    }

    /// Add a column to update if `value` is `Some`
    ///
    /// Can be called multiple times.
    pub fn set_if<I>(self, field: FieldProxy<I>, value: Option<<I::Field as Field>::Type>) -> Self
    where
        I: FieldProxyImpl<Field: SingleColumnField, Path = M>,
    {
        if let Some(value) = value {
            self.set(field, value)
        } else {
            self
        }
    }
}

impl<'ex, 'rf, E, M> UpdateBuilder<'rf, E, M, columns::NonEmpty>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Update a single row identified by a patch instance
    ///
    /// Note: The patch only provides the primary key, its other values will be ignored.
    pub async fn single<P>(self, patch: &P) -> Result<u64, Error>
    where
        P: Patch<Model = M> + Identifiable,
    {
        self.condition(patch.as_condition()).await
    }

    /// Update a bulk of rows identified by patch instances
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
        if let Some(condition) = DynamicCollection::or(conditions) {
            self.condition(condition).await
        } else {
            Ok(0)
        }
    }

    /// Update all rows matching a condition
    pub async fn condition<C: Condition<'rf>>(self, condition: C) -> Result<u64, Error> {
        let mut context = QueryContext::new();
        let columns: Vec<_> = self
            .columns
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_sql()))
            .collect();
        let condition_index = context.add_condition(&condition);
        let condition = context.get_condition(condition_index);
        database::update(self.executor, M::TABLE, &columns, Some(&condition)).await
    }

    /// Update all rows
    pub async fn all(self) -> Result<u64, Error> {
        let columns: Vec<_> = self
            .columns
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_sql()))
            .collect();
        database::update(self.executor, M::TABLE, &columns, None).await
    }
}
