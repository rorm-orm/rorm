//! Insert builder and macro

use std::marker::PhantomData;

use rorm_db::transaction::Transaction;
use rorm_db::{error::Error, Database, Row};

use crate::conditions::Value;
use crate::crud::builder::TransactionMarker;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::FieldProxy;
use crate::internal::field::{Field, RawField};
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
pub struct InsertBuilder<'db, 'rf, P, T, R> {
    db: &'db Database,
    transaction: T,
    returning: R,

    _phantom: PhantomData<&'rf P>,
}

impl<'db, 'rf, P> InsertBuilder<'db, 'rf, P, (), PrimaryKey<P::Model>>
where
    P: Patch,
{
    /// Start building a insert query
    pub fn new(db: &'db Database) -> Self {
        InsertBuilder {
            db,
            transaction: (),
            returning: PrimaryKey::new(),

            _phantom: PhantomData,
        }
    }
}

impl<'db, 'rf, P, R> InsertBuilder<'db, 'rf, P, (), R> {
    /// Add a transaction to the insert query
    pub fn transaction(
        self,
        transaction: &'rf mut Transaction<'db>,
    ) -> InsertBuilder<'db, 'rf, P, &'rf mut Transaction<'db>, R> {
        #[rustfmt::skip]
        let InsertBuilder { db, returning, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { db, transaction, _phantom, returning };
    }
}

impl<'db, 'rf, P, T, M> InsertBuilder<'db, 'rf, P, T, PrimaryKey<M>>
where
    M: Model,
    P: Patch<Model = M>,
{
    /// Set a tuple of fields to be returned after performing the insert
    ///
    /// ```skip
    /// let (some_field, another_field) = insert!(db, SomePatch)
    ///     .return_tuple((SomeModel::F.some_field, SomeModel::F.another_field))
    ///     .single(&some_patch)
    ///     .await?;
    /// ```
    pub fn return_tuple<Return, const C: usize>(
        self,
        tuple: Return,
    ) -> InsertBuilder<'db, 'rf, P, T, ReturnTuple<Return, C>>
    where
        ReturnTuple<Return, C>: Returning<M> + From<Return>,
    {
        #[rustfmt::skip]
        let InsertBuilder { db, transaction, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { db, transaction, returning: tuple.into(), _phantom };
    }

    /// Set a patch to be returned after performing the insert
    ///
    /// ```skip
    /// let another_patch = insert!(db, SomePatch)
    ///     .return_patch::<AnotherPatch>()
    ///     .single(&some_patch)
    ///     .await?;
    /// ```
    pub fn return_patch<Return>(self) -> InsertBuilder<'db, 'rf, P, T, ReturnPatch<Return>>
    where
        Return: Patch<Model = M>,
    {
        #[rustfmt::skip]
        let InsertBuilder { db, transaction, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { db, transaction, returning: ReturnPatch::new(), _phantom };
    }
}

impl<'db, 'rf, P, T, R> InsertBuilder<'db, 'rf, P, T, R>
where
    'db: 'rf,
    P: Patch,
    T: TransactionMarker<'rf, 'db>,
    R: Returning<P::Model>,
{
    /// Insert a single patch into the db
    pub async fn single(self, patch: &'rf P) -> Result<R::Result, Error> {
        let values = Vec::from_iter(iter_columns(patch).map(Value::into_sql));
        let columns = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        self.db
            .insert_returning(
                P::Model::TABLE,
                &columns,
                &values,
                self.transaction.into_option(),
                self.returning.columns(),
            )
            .await
            .and_then(R::decode)
    }

    /// Insert a bulk of patches into the db
    pub async fn bulk(
        self,
        patches: impl IntoIterator<Item = &'rf P>,
    ) -> Result<Vec<R::Result>, Error> {
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
                self.returning.columns(),
            )
            .await
            .and_then(|rows| {
                rows.into_iter()
                    .map(R::decode)
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
        $crate::crud::insert::InsertBuilder::<$patch, _, _>::new($db)
    };
}

/// Specifies which columns to return after a select and how to decode the rows into what.
pub trait Returning<M: Model> {
    /// Type as which rows should be decoded
    type Result;

    /// Decode a row
    fn decode(row: Row) -> Result<Self::Result, Error>;

    /// Columns to query
    fn columns(&self) -> &[&'static str];
}

/// The default [Returning] behaviour: return the primary key
pub struct PrimaryKey<M> {
    columns: [&'static str; 1],
    model: PhantomData<M>,
}
impl<M: Model> PrimaryKey<M> {
    const fn new() -> Self {
        Self {
            columns: [M::Primary::NAME],
            model: PhantomData,
        }
    }
}
impl<M: Model> Returning<M> for PrimaryKey<M> {
    type Result = <M::Primary as Field>::Type;

    fn decode(row: Row) -> Result<Self::Result, Error> {
        Ok(<M::Primary as Field>::Type::from_primitive(row.get(0)?))
    }

    fn columns(&self) -> &[&'static str] {
        &self.columns
    }
}

/// The [Returning] for patches.
pub struct ReturnPatch<P: Patch> {
    patch: PhantomData<P>,
    columns: Vec<&'static str>,
}
impl<P: Patch> ReturnPatch<P> {
    /// Create a SelectPatch
    pub fn new() -> Self {
        Self {
            patch: PhantomData,
            columns: P::COLUMNS.iter().flatten().copied().collect(),
        }
    }
}
impl<P: Patch> Default for ReturnPatch<P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<M: Model, P: Patch<Model = M>> Returning<M> for ReturnPatch<P> {
    type Result = P;

    fn decode(row: Row) -> Result<Self::Result, Error> {
        P::from_row_using_position(row)
    }

    fn columns(&self) -> &[&'static str] {
        &self.columns
    }
}

/// The [Returning] for tuple
///
/// Implemented for tuple of size 8 or less.
pub struct ReturnTuple<T, const C: usize> {
    #[allow(dead_code)]
    tuple: T,
    columns: [&'static str; C],
}
macro_rules! impl_select_tuple {
    ($C:literal, ($($F:ident @ $i:literal),+)) => {
        impl<$($F: Field),+> From<($(FieldProxy<$F, $F::Model>,)+)> for ReturnTuple<($(FieldProxy<$F, $F::Model>,)+), $C> {
            fn from(tuple: ($(FieldProxy<$F, $F::Model>,)+)) -> Self {
                Self {
                    tuple,
                    columns: [$(
                        $F::NAME,
                    )+],
                }
            }
        }
        impl<M: Model, $($F: Field<Model = M>),+> Returning<M> for ReturnTuple<($(FieldProxy<$F, M>,)+), $C>
        {
            type Result = ($(
                $F::Type,
            )+);

            fn decode(row: Row) -> Result<Self::Result, Error> {
                Ok(($(
                    $F::Type::from_primitive(row.get($i)?),
                )+))
            }

            fn columns(&self) -> &[&'static str] {
                &self.columns
            }
        }
    };
}
impl_select_tuple!(1, (A @ 0));
impl_select_tuple!(2, (A @ 0, B @ 1));
impl_select_tuple!(3, (A @ 0, B @ 1, C @ 2));
impl_select_tuple!(4, (A @ 0, B @ 1, C @ 2, D @ 3));
impl_select_tuple!(5, (A @ 0, B @ 1, C @ 2, D @ 3, E @ 4));
impl_select_tuple!(6, (A @ 0, B @ 1, C @ 2, D @ 3, E @ 4, F @ 5));
impl_select_tuple!(7, (A @ 0, B @ 1, C @ 2, D @ 3, E @ 4, F @ 5, G @ 6));
impl_select_tuple!(8, (A @ 0, B @ 1, C @ 2, D @ 3, E @ 4, F @ 5, G @ 6, H @ 7));
