//! Insert builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::Value;
use crate::model::{iter_columns, Model, Patch};

/// Builder for insert queries
///
/// Is is recommended to start a builder using [insert!](macro@crate::insert).
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
/// - `P`: [`Patch`](Patch)
///
///     The patches to insert.
///
/// - `T`: [`TransactionMarker<'rf,' db>`](TransactionMarker)
///
///     An optional transaction to execute this query in.
///
#[must_use]
pub struct InsertBuilder<'rf, E, P, R> {
    executor: E,
    returning: R,

    _phantom: PhantomData<&'rf P>,
}

impl<'ex, 'rf, E, P> InsertBuilder<'rf, E, P, returning::Patch<P::Model>>
where
    E: Executor<'ex>,
    P: Patch,
{
    /// Start building a insert query
    pub fn new(executor: E) -> Self {
        InsertBuilder {
            executor,
            returning: returning::Patch::new(),

            _phantom: PhantomData,
        }
    }
}

impl<'rf, E, P, M> InsertBuilder<'rf, E, P, returning::Patch<M>>
where
    M: Model,
    P: Patch<Model = M>,
{
    /// Remove the return value from the insert query reducing query time.
    pub fn return_nothing(self) -> InsertBuilder<'rf, E, P, returning::Nothing> {
        #[rustfmt::skip]
        let InsertBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { executor, returning: returning::Nothing, _phantom };
    }

    /// Remove the return value from the insert query reducing query time.
    pub fn return_primary_key(self) -> InsertBuilder<'rf, E, P, returning::PrimaryKey<M>> {
        #[rustfmt::skip]
        let InsertBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { executor, returning: returning::PrimaryKey::new(), _phantom };
    }

    /// Set a tuple of fields to be returned after performing the insert
    pub fn return_tuple<Return, const C: usize>(
        self,
        tuple: Return,
    ) -> InsertBuilder<'rf, E, P, returning::Tuple<Return, C>>
    where
        returning::Tuple<Return, C>: returning::Returning<M> + From<Return>,
    {
        #[rustfmt::skip]
        let InsertBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { executor, returning: tuple.into(), _phantom };
    }

    /// Set a patch to be returned after performing the insert
    pub fn return_patch<Return>(self) -> InsertBuilder<'rf, E, P, returning::Patch<Return>>
    where
        Return: Patch<Model = M>,
    {
        #[rustfmt::skip]
        let InsertBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { executor, returning: returning::Patch::new(), _phantom };
    }
}

impl<'ex, 'rf, E, P, R> InsertBuilder<'rf, E, P, R>
where
    E: Executor<'ex>,
    P: Patch,
    R: returning::Returning<P::Model>,
{
    /// Insert a single patch into the db
    pub async fn single(self, patch: &'rf P) -> Result<R::Result, Error> {
        let values = Vec::from_iter(iter_columns(patch).map(Value::into_sql));
        let inserting = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        let returning = self.returning.columns();

        if returning.is_empty() {
            database::insert(self.executor, P::Model::TABLE, &inserting, &values).await?;
            R::produce_result()
        } else {
            database::insert_returning(
                self.executor,
                P::Model::TABLE,
                &inserting,
                &values,
                returning,
            )
            .await
            .and_then(R::decode)
        }
    }

    /// Insert a bulk of patches into the db
    pub async fn bulk(
        self,
        patches: impl IntoIterator<Item = &'rf P>,
    ) -> Result<R::BulkResult, Error> {
        let mut values = Vec::new();
        for patch in patches {
            values.push(Vec::from_iter(iter_columns(patch).map(Value::into_sql)));
        }
        let values_slices = Vec::from_iter(values.iter().map(Vec::as_slice));
        let inserting = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        let returning = self.returning.columns();

        if returning.is_empty() {
            database::insert_bulk(self.executor, P::Model::TABLE, &inserting, &values_slices)
                .await?;
            R::produce_result_bulk()
        } else {
            database::insert_bulk_returning(
                self.executor,
                P::Model::TABLE,
                &inserting,
                &values_slices,
                returning,
            )
            .await
            .and_then(R::decode_bulk)
        }
    }
}

/// Create an INSERT query.
///
/// # Basic usage
/// ```no_run
/// # use rorm::{Model, Patch, Database, insert};
/// # #[derive(Model)] pub struct User { #[rorm(id)] id: i64, #[rorm(max_length = 255)] name: String, }
/// # #[derive(Patch)] #[rorm(model = "User")] pub struct NewUser { name: String, }
/// pub async fn create_single_user(db: &Database, user: &NewUser) {
///     insert!(db, NewUser)
///         .single(user)
///         .await
///         .unwrap();
/// }
/// pub async fn create_many_users(db: &Database, users: &[NewUser]) {
///     insert!(db, NewUser)
///         .bulk(users)
///         .await
///         .unwrap();
/// }
///```
///
/// Like every crud macro `insert!` starts a [builder](InsertBuilder) which is consumed to execute the query.
///
/// `insert!`'s first argument is a reference to the [`Database`].
/// Its second is the [`Patch`] type you want to insert instances of.
///
/// Since your [`Model`] type will probably contain a primary key which is set by the database,
/// you'll rarely insert your actual model instances.
///
/// To specify the patch instances use the method [`single`](InsertBuilder::single) or
/// [`bulk`](InsertBuilder::bulk), which will consume the builder and execute the query.
///
/// # Return value
/// ```no_run
/// # use rorm::{Model, Patch, Database, insert, Error};
/// # #[derive(Model)] pub struct User { #[rorm(id)] id: i64, #[rorm(max_length = 255)] name: String, }
/// # #[derive(Patch)] #[rorm(model = "User")] pub struct NewUser { name: String, }
/// # pub type UserPatch = NewUser;
/// pub async fn show_various_returns(db: &Database, user: &NewUser) -> Result<(), Error> {
///     // Return model instance by default
///     let _: User = insert!(db, NewUser)
///         .single(user)
///         .await?;
///
///     // Return any patch instance (including the one used to insert and the model itself)
///     let _: UserPatch = insert!(db, NewUser)
///         .return_patch::<UserPatch>() // turbo fish not necessarily required but more readable
///         .single(user)
///         .await?;
///
///     // Return a tuple of fields
///     let _: (i64, String) = insert!(db, NewUser)
///         .return_tuple((User::F.id, User::F.name))
///         .single(user)
///         .await?;
///
///     // Return the model's primary key
///     let _: i64 = insert!(db, NewUser)
///         .return_primary_key()
///         .single(user)
///         .await?;
///
///     // Return nothing
///     let _: () = insert!(db, NewUser)
///         .return_nothing()
///         .single(user)
///         .await?;
///
///     Ok(())
/// }
///```
#[macro_export]
macro_rules! insert {
    ($db:expr, $patch:path) => {
        $crate::crud::insert::InsertBuilder::<_, $patch, _>::new($db)
    };
}

#[doc(hidden)]
pub mod returning {
    use std::marker::PhantomData;

    use crate::error::Error;
    use crate::internal::field::as_db_type::AsDbType;
    use crate::internal::field::{Field, FieldProxy, RawField};
    use crate::model::{Model, Patch as ModelPatch};
    use crate::row::Row;

    /// Specifies which columns to return after a select and how to decode the rows into what.
    pub trait Returning<M: Model> {
        /// Type as which rows should be decoded
        type Result;

        /// Type as which lists of rows should be decoded
        ///
        /// This defaults to `Vec<Self::Result>`.
        type BulkResult;

        /// Produce a result when [Self::columns] returns an empty slice and no actual row could be retrieved.
        fn produce_result() -> Result<Self::Result, Error> {
            Err(Error::DecodeError("No columns where specified".to_string()))
        }

        /// Produce a bulk result when [Self::columns] returns an empty slice and no actual rows could be retrieved.
        fn produce_result_bulk() -> Result<Self::BulkResult, Error> {
            Err(Error::DecodeError("No columns where specified".to_string()))
        }

        /// Decode a single row
        fn decode(row: Row) -> Result<Self::Result, Error>;

        /// Decode many rows
        ///
        /// This default to `rows.into_iter().map(Self::decode).collect()`.
        fn decode_bulk(rows: Vec<Row>) -> Result<Self::BulkResult, Error>;

        /// Columns to query
        fn columns(&self) -> &[&'static str];
    }

    /// Add the default implementation for the bulk members of [Returning]
    /// i.e. just apply the single impl to a [Vec].
    macro_rules! default_bulk_impl {
        () => {
            type BulkResult = Vec<Self::Result>;

            fn decode_bulk(rows: Vec<Row>) -> Result<Self::BulkResult, Error> {
                rows.into_iter().map(Self::decode).collect()
            }
        };
    }

    /// The [Returning] for nothing.
    pub struct Nothing;

    impl<M: Model> Returning<M> for Nothing {
        type Result = ();
        type BulkResult = ();

        fn produce_result() -> Result<Self::Result, Error> {
            Ok(())
        }

        fn produce_result_bulk() -> Result<Self::BulkResult, Error> {
            Ok(())
        }

        fn decode(_row: Row) -> Result<Self::Result, Error> {
            unreachable!("returning::Nothing::columns should return an empty slice")
        }

        fn decode_bulk(_rows: Vec<Row>) -> Result<Self::BulkResult, Error> {
            unreachable!("returning::Nothing::columns should return an empty slice")
        }

        fn columns(&self) -> &[&'static str] {
            &[]
        }
    }

    /// The default [Returning] behaviour: return the primary key
    pub struct PrimaryKey<M> {
        columns: [&'static str; 1],
        model: PhantomData<M>,
    }

    impl<M: Model> PrimaryKey<M> {
        pub(crate) const fn new() -> Self {
            Self {
                columns: [M::Primary::NAME],
                model: PhantomData,
            }
        }
    }

    impl<M: Model> Returning<M> for PrimaryKey<M> {
        default_bulk_impl!();

        type Result = <M::Primary as Field>::Type;

        fn decode(row: Row) -> Result<Self::Result, Error> {
            Ok(<M::Primary as Field>::Type::from_primitive(row.get(0)?))
        }

        fn columns(&self) -> &[&'static str] {
            &self.columns
        }
    }

    /// The [Returning] for patches.
    pub struct Patch<P: ModelPatch> {
        patch: PhantomData<P>,
        columns: Vec<&'static str>,
    }

    impl<P: ModelPatch> Patch<P> {
        pub(crate) fn new() -> Self {
            Self {
                patch: PhantomData,
                columns: P::COLUMNS.iter().flatten().copied().collect(),
            }
        }
    }

    impl<M: Model, P: ModelPatch<Model = M>> Returning<M> for Patch<P> {
        default_bulk_impl!();

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
    pub struct Tuple<T, const C: usize> {
        #[allow(dead_code)]
        tuple: T,
        columns: [&'static str; C],
    }
    macro_rules! impl_select_tuple {
        ($C:literal, ($($F:ident @ $i:literal),+)) => {
            impl<$($F: Field),+> From<($(FieldProxy<$F, $F::Model>,)+)> for Tuple<($(FieldProxy<$F, $F::Model>,)+), $C> {
                fn from(tuple: ($(FieldProxy<$F, $F::Model>,)+)) -> Self {
                    Self {
                        tuple,
                        columns: [$(
                            $F::NAME,
                        )+],
                    }
                }
            }
            impl<M: Model, $($F: Field<Model = M>),+> Returning<M> for Tuple<($(FieldProxy<$F, M>,)+), $C> {
                default_bulk_impl!();

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
}
