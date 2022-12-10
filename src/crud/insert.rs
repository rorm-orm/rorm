//! Insert builder and macro

use crate::conditions::Value;
use crate::crud::builder::TransactionMarker;
use crate::internal::field::{AbstractField, Field, RawField};
use rorm_db::transaction::Transaction;
use rorm_db::{error::Error, Database};
use std::marker::PhantomData;

use crate::model::{iter_columns, Model, Patch};

/// Builder for insert queries
///
/// Is is recommended to start a builder using [insert!](macro@crate::insert).
///
/// ## Generics
/// - `'rf`
///
///     Lifetime of the transaction reference.
///
/// - `'db: 'rf`
///
///     The database reference's lifetime.
///     Since `'rf` applies to a transaction reference, `'db` must outlive `'rf`.
///
/// - `P`: [Patch](Patch)
///
///     The patches to insert.
///
/// - `T`: [TransactionMarker<'rf,' db>](TransactionMarker)
///
///     An optional transaction to execute this query in.
///
#[must_use]
pub struct InsertBuilder<'db, 'rf, P, T> {
    db: &'db Database,
    transaction: T,

    _phantom: PhantomData<&'rf P>,
}

impl<'db, 'rf, P> InsertBuilder<'db, 'rf, P, ()>
where
    P: Patch,
{
    /// Start building a insert query
    pub fn new(db: &'db Database) -> Self {
        InsertBuilder {
            db,
            transaction: (),

            _phantom: PhantomData,
        }
    }
}

impl<'db, 'rf, P> InsertBuilder<'db, 'rf, P, ()> {
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

impl<'db, 'rf, P, T> InsertBuilder<'db, 'rf, P, T>
where
    'db: 'rf,
    P: Patch,
    T: TransactionMarker<'rf, 'db>,
{
    /// Insert a single patch into the db
    pub async fn single(
        self,
        patch: &'rf P,
    ) -> Result<<<<P as Patch>::Model as Model>::Primary as Field>::Type, Error> {
        let values = Vec::from_iter(iter_columns(patch).map(Value::into_sql));
        let columns = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        self.db
            .insert_returning(
                P::Model::TABLE,
                &columns,
                &values,
                self.transaction.into_option(),
                &[<<P as Patch>::Model as Model>::Primary::NAME],
            )
            .await
            .and_then(|row| {
                <<P as Patch>::Model as Model>::Primary::get_from_row(&row, 0).map(Into::into)
            })
    }

    /// Insert a bulk of patches into the db
    pub async fn bulk(
        self,
        patches: impl IntoIterator<Item = &'rf P>,
    ) -> Result<Vec<<<<P as Patch>::Model as Model>::Primary as Field>::Type>, Error> {
        let mut values = Vec::new();
        for patch in patches {
            values.push(Vec::from_iter(iter_columns(patch).map(Value::into_sql)));
        }
        let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
        let columns = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        self.db
            .insert_bulk_returning(
                P::Model::TABLE,
                &columns,
                &values_slices,
                self.transaction.into_option(),
                &[<<P as Patch>::Model as Model>::Primary::NAME],
            )
            .await
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| {
                        <<P as Patch>::Model as Model>::Primary::get_from_row(&row, 0)
                            .map(Into::into)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
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
