//! Delete builder

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
    pub fn instance(self, model: &'rf M) -> DeleteBuilder<'db, 'rf, M, Condition<'rf>, T> {
        self.condition(
            model
                .as_condition()
                .expect("Model should always have a primary key"),
        )
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

/// Slightly less verbose macro to start a [`DeleteBuilder`] from a patch
#[macro_export]
macro_rules! delete {
    ($db:expr, $model:path) => {
        $crate::crud::delete::DeleteBuilder::<$model, _, _>::new($db)
    };
}
