//! Update builder and macro

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use rorm_db::error::Error;
use rorm_db::transaction::Transaction;
use rorm_db::value::Value;
use rorm_db::Database;

use crate::conditions::{Condition, IntoSingleValue};
use crate::crud::builder::{ConditionMarker, Sealed, TransactionMarker};
use crate::internal::field::{Field, FieldProxy};
use crate::internal::query_context::QueryContext;
use crate::Model;

/// Marker for the generic parameter storing a list of columns.
pub trait ColumnsMarker: Sealed {}

impl<'a> Sealed for Vec<(&'static str, Value<'a>)> {}
impl<'a> ColumnsMarker for Vec<(&'static str, Value<'a>)> {}
impl ColumnsMarker for () {}

/// Builder for update queries
///
/// Is is recommended to start a builder using [update!].
///
/// [update!]: macro@crate::update
#[must_use]
pub struct UpdateBuilder<
    'db: 'rf,
    'rf,
    M: Model,
    L: ColumnsMarker,
    C: ConditionMarker<'rf>,
    T: TransactionMarker<'rf, 'db>,
> {
    db: &'db Database,
    columns: L,
    condition: C,
    transaction: T,

    _phantom: PhantomData<&'rf M>,
}

impl<'db, 'rf, M: Model> UpdateBuilder<'db, 'rf, M, (), (), ()> {
    /// Start building a delete query
    pub fn new(db: &'db Database) -> Self {
        Self {
            db,
            columns: (),
            condition: (),
            transaction: (),

            _phantom: PhantomData,
        }
    }
}

impl<'db, 'rf, M: Model, L: ColumnsMarker, T: TransactionMarker<'rf, 'db>>
    UpdateBuilder<'db, 'rf, M, L, (), T>
{
    /// Add a condition to the query
    pub fn condition<C: Condition<'rf>>(self, condition: C) -> UpdateBuilder<'db, 'rf, M, L, C, T> {
        #[rustfmt::skip]
        let UpdateBuilder { db, columns, _phantom, transaction, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { db, columns, _phantom, condition, transaction, };
    }
}

impl<'db: 'rf, 'rf, M: Model, L: ColumnsMarker, C: ConditionMarker<'rf>>
    UpdateBuilder<'db, 'rf, M, L, C, ()>
{
    /// Add a transaction to the query
    pub fn transaction(
        self,
        transaction: &'rf mut Transaction<'db>,
    ) -> UpdateBuilder<'db, 'rf, M, L, C, &'rf mut Transaction<'db>> {
        #[rustfmt::skip]
        let UpdateBuilder { db, columns, _phantom, condition, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { db, columns, _phantom, condition, transaction, };
    }
}

impl<'db: 'rf, 'rf, M: Model, C: ConditionMarker<'rf>, T: TransactionMarker<'rf, 'db>>
    UpdateBuilder<'db, 'rf, M, (), C, T>
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<F: Field>(
        self,
        _field: FieldProxy<F, M>,
        value: impl IntoSingleValue<'rf, F::DbType>,
    ) -> UpdateBuilder<'db, 'rf, M, Vec<(&'static str, Value<'rf>)>, C, T> {
        #[rustfmt::skip]
        let UpdateBuilder { db, _phantom, condition, transaction, .. } = self;
        #[rustfmt::skip]
        return UpdateBuilder { db, columns: vec![(F::NAME, value.into_value())], _phantom, condition, transaction, };
    }
}

impl<'db: 'rf, 'rf, M: Model, C: ConditionMarker<'rf>, T: TransactionMarker<'rf, 'db>>
    UpdateBuilder<'db, 'rf, M, Vec<(&'static str, Value<'rf>)>, C, T>
{
    /// Add a column to update.
    ///
    /// Can be called multiple times.
    pub fn set<F: Field>(
        self,
        _field: FieldProxy<F, M>,
        value: impl IntoSingleValue<'rf, F::DbType>,
    ) -> Self {
        let mut builder = self;
        builder.columns.push((F::NAME, value.into_value()));
        builder
    }

    /// Perform the update operation
    pub async fn exec(self) -> Result<u64, Error> {
        let context = QueryContext::new();
        self.db
            .update(
                M::TABLE,
                &self.columns,
                self.condition.into_option(&context).as_ref(),
                self.transaction.into_option(),
            )
            .await
    }
}

impl<'db, 'rf, M: Model + 'rf, C: ConditionMarker<'rf>, T: TransactionMarker<'rf, 'db>> IntoFuture
    for UpdateBuilder<'db, 'rf, M, Vec<(&'static str, Value<'rf>)>, C, T>
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
/// 4. *Optionally* add this query to a transaction
///
///     `.transaction(&mut tr)`
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
        $crate::crud::update::UpdateBuilder::<$model, _, _, _>::new($db)
    };
}
