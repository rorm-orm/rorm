//! Insert builder and macro

use crate::conditions::Value;
use crate::crud::builder::TransactionMarker;
use rorm_db::transaction::Transaction;
use rorm_db::{error::Error, Database};
use std::marker::PhantomData;

use crate::model::{iter_columns, Model, Patch};

/// Builder for insert queries
///
/// Is is recommended to start a builder using [insert!].
///
/// [insert!]: macro@crate::insert
#[must_use]
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
        let values = Vec::from_iter(iter_columns(patch).map(Value::into_sql));
        self.db
            .insert(
                P::Model::TABLE,
                P::COLUMNS,
                &values,
                self.transaction.into_option(),
            )
            .await
    }

    /// Insert a bulk of patches into the db
    pub async fn bulk(self, patches: impl IntoIterator<Item = &'rf P>) -> Result<(), Error> {
        let mut values = Vec::new();
        for patch in patches {
            values.push(Vec::from_iter(iter_columns(patch).map(Value::into_sql)));
        }
        let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
        self.db
            .insert_bulk(
                P::Model::TABLE,
                P::COLUMNS,
                &values_slices,
                self.transaction.into_option(),
            )
            .await
    }
}

/// Create an INSERT query.
///
/// 1. Give a reference to your db and the patch type you want to insert instances of.
///
///     `insert!(&db, MyPatchType)`
///
/// 2. *Optionally* add this query to a transaction
///
///     `.transaction(&mut tr)`
///
/// 3. Execute the actual query with ether a [single] instance or in a [bulk].
///
///     `.single(&my_instance)`
///
///     `.bulk(&[one_instances, another_instance])`
///
/// Example:
/// ```no_run
/// # use rorm::{Model, Patch, Database, insert};
/// #
/// # #[derive(Model)]
/// # struct User {
/// #     #[rorm(id)]
/// #     id: i64,
/// #
/// #     #[rorm(max_length = 255)]
/// #     name: String,
/// # }
/// #
/// # #[derive(Patch)]
/// # #[rorm(model = "User")]
/// # struct NewUser {
/// #     name: String
/// # }
/// #
/// pub async fn create_user(db: &Database, name: String) {
///     insert!(db, NewUser)
///         .single(&NewUser { name })
///         .await
///         .unwrap();
/// }
/// ```
///
/// [single]: InsertBuilder::single
/// [bulk]: InsertBuilder::bulk
#[macro_export]
macro_rules! insert {
    ($db:expr, $patch:path) => {
        $crate::crud::insert::InsertBuilder::<$patch, _>::new($db)
    };
}
