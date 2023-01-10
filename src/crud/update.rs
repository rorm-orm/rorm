//! Update builder and macro

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;
use rorm_db::value::Value;

use crate::conditions::{Condition, IntoSingleValue};
use crate::crud::builder::ConditionMarker;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{Field, FieldProxy};
use crate::internal::query_context::QueryContext;
use crate::Model;

/// Wrapper around `Vec` to indicate on type level, that possible no column has been set yet.
pub struct OptionalColumns<'a>(Vec<(&'static str, Value<'a>)>);

/// Builder for update queries
///
/// Is is recommended to start a builder using [update!](macro@crate::update).
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
///     The model from whose table to update rows.
///
/// - `L`
///
///     List of columns and values to set.
///     This is a generic instead of just being a `Vec` in order to prevent the list from being empty.
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
    /// Call [finish_dyn_set](UpdateBuilder::finish_dyn_set) to go back to normal operation.
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
        value: impl IntoSingleValue<'rf, <<F as Field>::Type as AsDbType>::DbType<F>, F>,
    ) -> Self {
        let mut builder = self;
        builder.columns.0.push((F::NAME, value.into_value()));
        builder
    }

    /// Go back to a "normal" builder after calling [begin_dyn_set](UpdateBuilder::begin_dyn_set).
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
        value: impl IntoSingleValue<'rf, <<F as Field>::Type as AsDbType>::DbType<F>, F>,
    ) -> UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C> {
        #[rustfmt::skip]
        let UpdateBuilder { executor, _phantom, condition, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { executor, columns: vec![(F::NAME, value.into_value())], _phantom, condition, };
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
        value: impl IntoSingleValue<'rf, <<F as Field>::Type as AsDbType>::DbType<F>, F>,
    ) -> Self {
        let mut builder = self;
        builder.columns.push((F::NAME, value.into_value()));
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
        database::update(
            self.executor,
            M::TABLE,
            &self.columns,
            self.condition.into_option(&context).as_ref(),
        )
        .await
    }
}

impl<'rf, E, M, C> IntoFuture for UpdateBuilder<'rf, E, M, Vec<(&'static str, Value<'rf>)>, C>
where
    E: Executor<'rf> + 'rf,
    M: Model,
    C: ConditionMarker<'rf>,
{
    type Output = Result<u64, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'rf>>;

    /// Convert a [UpdateBuilder] with columns into a [Future] implicitly
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

/// Create a UPDATE query.
///
/// 1. Give a reference to your db and the patch type you want to update instances of
///
///     `update!(&db, MyModelType)`
///
/// 2. Set some columns to update
///
///     `.set(MyModelType::F.some_field, 3)`
///
///     `.set(MyModelType::F.some_other_field, "hi")`
///
/// 3. Restrict what rows to update with a condition
///
///     `.condition(MyModelType::F.id.greater(0))`
///
/// 5. Execute. After step 2 you could already `.await`ed your query.
///
/// Example:
/// ```no_run
/// # use rorm::{Model, Database, update};
/// #
/// # #[derive(Model)]
/// # struct User {
/// #     #[rorm(id)]
/// #     id: i64,
/// #
/// #     #[rorm(max_length = 255)]
/// #     password: String,
/// # }
/// #
/// pub async fn set_good_password(db: &Database) {
///     update!(db, User)
///         .set(User::F.password, "I am way more secure™")
///         .condition(User::F.password.equals("password"))
///         .await
///         .unwrap();
/// }
/// ```
#[macro_export]
macro_rules! update {
    ($db:expr, $model:path) => {
        $crate::crud::update::UpdateBuilder::<_, $model, _, _>::new($db)
    };
}
