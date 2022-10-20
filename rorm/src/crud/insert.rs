//! Insert functions

use crate::crud::builder::TransactionMarker;
use rorm_db::transaction::Transaction;
use rorm_db::{error::Error, Database};
use std::marker::PhantomData;

use crate::model::{iter_columns, Model, Patch};

/// Builder for insert queries
pub struct InsertBuilder<'db: 'rf, 'rf, P: Patch, T: TransactionMarker<'rf, 'db>> {
    db: &'db Database,
    transaction: T,

    _phantom: PhantomData<&'rf P>,
}

impl<'db: 'rf, 'rf, P: Patch> InsertBuilder<'db, 'rf, P, ()> {
    /// Start building a insert query
    pub fn new(db: &'db Database) -> Self {
        InsertBuilder {
            db,
            transaction: (),

            _phantom: PhantomData,
        }
    }
}

impl<'db: 'rf, 'rf, P: Patch> InsertBuilder<'db, 'rf, P, ()> {
    /// Add a transaction to the insert query
    pub fn transaction(
        self,
        transaction: &'rf mut Transaction<'db>,
    ) -> InsertBuilder<'db, 'rf, P, &'rf mut Transaction<'db>> {
        #[rustfmt::skip]
        let InsertBuilder { db, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { db, transaction, _phantom, };
    }
}

impl<'db: 'rf, 'rf, P: Patch, T: TransactionMarker<'rf, 'db>> InsertBuilder<'db, 'rf, P, T> {
    /// Insert a single patch into the db
    pub async fn single(self, patch: &'rf P) -> Result<(), Error> {
        let values = Vec::from_iter(iter_columns(patch));
        self.db
            .insert(
                P::Model::table_name(),
                P::COLUMNS,
                &values,
                self.transaction.into_option(),
            )
            .await
    }

    /// Insert a bulk of patches into the db
    pub async fn insert_bulk(self, patches: impl IntoIterator<Item = &'rf P>) -> Result<(), Error> {
        let mut values = Vec::new();
        for patch in patches {
            values.push(Vec::from_iter(iter_columns(patch)));
        }
        let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
        self.db
            .insert_bulk(
                P::Model::table_name(),
                P::COLUMNS,
                &values_slices,
                self.transaction.into_option(),
            )
            .await
    }
}

/// Slightly less verbose macro to start a [`InsertBuilder`] from a patch
#[macro_export]
macro_rules! insert {
    ($db:expr, $model:path) => {
        $crate::crud::insert::InsertBuilder::<$model, _>::new($db)
    };
}
