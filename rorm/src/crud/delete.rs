//! Delete builder and macro

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

use rorm_db::error::Error;
use rorm_db::transaction::Transaction;
use rorm_db::Database;

use crate::conditional::Condition;
use crate::crud::builder::{ConditionMarker, TransactionMarker};
use crate::model::Model;

/// Builder for delete queries
///
/// Is is recommended to start a builder using [delete!].
///
/// [delete!]: macro@crate::delete
#[must_use]
pub struct DeleteBuilder<
    'db: 'rf,
    'rf,
    M: Model,
    C: ConditionMarker<'rf>,
    T: TransactionMarker<'rf, 'db>,
> {
    db: &'db Database,
    condition: C,
    transaction: T,

    _phantom: PhantomData<&'rf M>,
}

impl<'db, 'rf, M: Model> DeleteBuilder<'db, 'rf, M, (), ()> {
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

impl<'db, 'rf, M: Model, T: TransactionMarker<'rf, 'db>> DeleteBuilder<'db, 'rf, M, (), T> {
    /// Set the condition to delete a single model instance
    pub fn single(self, model: &'rf M) -> DeleteBuilder<'db, 'rf, M, Condition<'rf>, T> {
        self.condition(
            model
                .as_condition()
                .expect("Model should always have a primary key"),
        )
    }

    /// Set the condition to delete a bulk of model instances
    pub fn bulk(
        self,
        models: impl IntoIterator<Item = &'rf M>,
    ) -> DeleteBuilder<'db, 'rf, M, Condition<'rf>, T> {
        self.condition(Condition::Disjunction(
            models
                .into_iter()
                .map(|model| {
                    model
                        .as_condition()
                        .expect("Model should always have a primary key")
                })
                .collect(),
        ))
    }

    /// Add a condition to the delete query
    pub fn condition(
        self,
        condition: Condition<'rf>,
    ) -> DeleteBuilder<'db, 'rf, M, Condition<'rf>, T> {
        #[rustfmt::skip]
        let DeleteBuilder { db, transaction, _phantom, .. } = self;
        #[rustfmt::skip]
        return DeleteBuilder { db, condition, transaction, _phantom, };
    }
}

impl<'db, 'rf, M: Model, C: ConditionMarker<'rf>> DeleteBuilder<'db, 'rf, M, C, ()> {
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

impl<'db, 'rf, M: Model, T: TransactionMarker<'rf, 'db>> DeleteBuilder<'db, 'rf, M, (), T> {
    /// Delete all columns
    pub async fn all(self) -> Result<u64, Error> {
        self.db
            .delete(M::table_name(), None, self.transaction.into_option())
            .await
    }
}

impl<'db, 'rf, M: Model, C: ConditionMarker<'rf>, T: TransactionMarker<'rf, 'db>>
    DeleteBuilder<'db, 'rf, M, C, T>
{
    /// Perform the delete operation
    async fn exec(self) -> Result<u64, Error> {
        self.db
            .delete(
                M::table_name(),
                self.condition.as_option(),
                self.transaction.into_option(),
            )
            .await
    }
}

impl<'db, 'rf, M: Model + 'rf, T: TransactionMarker<'rf, 'db>> IntoFuture
    for DeleteBuilder<'db, 'rf, M, Condition<'rf>, T>
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
