//! Insert builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::Value;
use crate::model::{Model, Patch};

/// Builder for insert queries
///
/// Is is recommended to start a builder using [`insert!`](macro@crate::insert).
///
/// ## Generics
/// - `E`: [`Executor`]
///
///     The executor to query with.
///
/// - `M`: [`Model`]
///
///     The model into whose table to insert rows.
///
/// - `R`: [`Returning<P::Model>`](returning::Returning)
///
///     What to return after the insert.
///
#[must_use]
pub struct InsertBuilder<E, M, R> {
    executor: E,
    returning: R,

    _phantom: PhantomData<M>,
}

impl<'ex, E, M> InsertBuilder<E, M, returning::Patch<M>>
where
    E: Executor<'ex>,
    M: Model,
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

impl<E, M> InsertBuilder<E, M, returning::Patch<M>>
where
    M: Model,
{
    fn set_return<R>(self, returning: R) -> InsertBuilder<E, M, R> {
        #[rustfmt::skip]
        let InsertBuilder { executor, _phantom, .. } = self;
        #[rustfmt::skip]
        return InsertBuilder { executor, returning, _phantom };
    }

    /// Remove the return value from the insert query reducing query time.
    pub fn return_nothing(self) -> InsertBuilder<E, M, returning::Nothing> {
        self.set_return(returning::Nothing)
    }

    /// Remove the return value from the insert query reducing query time.
    pub fn return_primary_key(self) -> InsertBuilder<E, M, returning::PrimaryKey<M>> {
        self.set_return(returning::PrimaryKey::new())
    }

    /// Set a tuple of fields to be returned after performing the insert
    pub fn return_tuple<Return, const C: usize>(
        self,
        tuple: Return,
    ) -> InsertBuilder<E, M, returning::Tuple<Return, C>>
    where
        returning::Tuple<Return, C>: returning::Returning<M> + From<Return>,
    {
        self.set_return(tuple.into())
    }

    /// Set a patch to be returned after performing the insert
    pub fn return_patch<Return>(self) -> InsertBuilder<E, M, returning::Patch<Return>>
    where
        Return: Patch<Model = M>,
    {
        self.set_return(returning::Patch::new())
    }
}

impl<'ex, E, M, R> InsertBuilder<E, M, R>
where
    E: Executor<'ex>,
    M: Model,
    R: returning::Returning<M>,
{
    /// Insert a single patch into the db
    pub async fn single<P: Patch<Model = M>>(self, patch: &P) -> Result<R::Result, Error> {
        let values = patch.values();
        let values: Vec<_> = values.iter().map(Value::as_sql).collect();
        let inserting: Vec<_> = P::COLUMNS.iter().flatten().cloned().collect();
        let returning = self.returning.columns();

        if returning.is_empty() {
            database::insert(self.executor, M::TABLE, &inserting, &values).await?;
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
    pub async fn bulk<P: Patch<Model = M>>(
        self,
        patches: impl IntoIterator<Item = &P>,
    ) -> Result<R::BulkResult, Error> {
        let num_cols = P::COLUMNS.iter().filter(|o| o.is_some()).count();

        let mut values = Vec::new();
        for patch in patches {
            values.extend(patch.values());
        }

        let values: Vec<_> = values.iter().map(Value::as_sql).collect();
        let values_slices: Vec<_> = values.chunks(num_cols).collect();
        let inserting = Vec::from_iter(P::COLUMNS.iter().flatten().cloned());
        let returning = self.returning.columns();

        if returning.is_empty() {
            database::insert_bulk(self.executor, M::TABLE, &inserting, &values_slices).await?;
            R::produce_result_bulk()
        } else {
            database::insert_bulk_returning(
                self.executor,
                M::TABLE,
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
/// `insert!`'s first argument is a reference to the [`Database`](crate::Database).
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
        $crate::crud::insert::InsertBuilder::<_, <$patch as $crate::model::Patch>::Model, _>::new(
            $db,
        )
    };
}

#[doc(hidden)]
pub mod returning {
    use std::marker::PhantomData;

    use crate::error::Error;
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

        /// Produce a result when [`Self::columns`] returns an empty slice and no actual row could be retrieved.
        fn produce_result() -> Result<Self::Result, Error> {
            Err(Error::DecodeError("No columns where specified".to_string()))
        }

        /// Produce a bulk result when [`Self::columns`] returns an empty slice and no actual rows could be retrieved.
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

    /// Add the default implementation for the bulk members of [`Returning`]
    /// i.e. just apply the single impl to a [`Vec`].
    macro_rules! default_bulk_impl {
        () => {
            type BulkResult = Vec<Self::Result>;

            fn decode_bulk(rows: Vec<Row>) -> Result<Self::BulkResult, Error> {
                rows.into_iter().map(Self::decode).collect()
            }
        };
    }

    /// The [`Returning`] for nothing.
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

    /// The default [`Returning`] behaviour: return the primary key
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

        type Result = <M::Primary as RawField>::Type;

        fn decode(row: Row) -> Result<Self::Result, Error> {
            Ok(<M::Primary as Field>::from_primitive(row.get(0)?))
        }

        fn columns(&self) -> &[&'static str] {
            &self.columns
        }
    }

    /// The [`Returning`] for patches.
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

    /// The [`Returning`] for tuple
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
                        $F::from_primitive(row.get($i)?),
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
