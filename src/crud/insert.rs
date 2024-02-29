//! Insert builder and macro

use std::marker::PhantomData;

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::Executor;

use crate::conditions::Value;
use crate::crud::decoder::Decoder;
use crate::crud::selector::Selector;
use crate::internal::field::FieldProxy;
use crate::internal::patch::{IntoPatchCow, PatchCow};
use crate::internal::query_context::QueryContext;
use crate::model::{Model, Patch, PatchSelector};

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
/// - `S`: [`Selector`]
///
///     What to return after the insert.
///
#[must_use]
pub struct InsertBuilder<E, M, S> {
    executor: E,
    selector: S,
    model: PhantomData<M>,
}

impl<'ex, E, M> InsertBuilder<E, M, PatchSelector<M, M>>
where
    E: Executor<'ex>,
    M: Model,
{
    /// Start building a insert query
    pub fn new(executor: E, _: M::InsertPermission) -> Self {
        InsertBuilder {
            executor,
            selector: PatchSelector::new(),
            model: PhantomData,
        }
    }

    fn set_return<S>(self, selector: S) -> InsertBuilder<E, M, S>
    where
        S: Selector<Model = M>,
    {
        InsertBuilder {
            executor: self.executor,
            selector,
            model: PhantomData,
        }
    }

    /// Remove the return value from the insert query reducing query time.
    pub fn return_nothing(self) -> InsertReturningNothing<E, M> {
        InsertReturningNothing {
            executor: self.executor,
            model: PhantomData,
        }
    }

    /// Return the tables primary key after performing the insert
    pub fn return_primary_key(self) -> InsertBuilder<E, M, FieldProxy<M::Primary, M>> {
        self.set_return(FieldProxy::new())
    }

    /// Set a tuple of fields to be returned after performing the insert
    pub fn return_tuple<Return>(self, tuple: Return) -> InsertBuilder<E, M, Return>
    where
        Return: Selector<Model = M>,
    {
        self.set_return(tuple)
    }

    /// Set a patch to be returned after performing the insert
    pub fn return_patch<Return>(self) -> InsertBuilder<E, M, PatchSelector<Return, M>>
    where
        Return: Patch<Model = M>,
    {
        self.set_return(PatchSelector::new())
    }
}

impl<'ex, E, M, S> InsertBuilder<E, M, S>
where
    E: Executor<'ex>,
    M: Model,
    S: Selector<Model = M>,
{
    // Until rust supports checking constants in type bounds, this ugly check is necessary
    const CHECK: () = {
        if !S::INSERT_COMPATIBLE {
            panic!("An invalid selector was passed to an InsertBuilder! Please check you're insert!(..).return_tuple(..) calls!");
        }
    };

    /// Insert a single patch into the db
    pub async fn single<P: Patch<Model = M>>(self, patch: &P) -> Result<S::Result, Error> {
        // it is intentional to force the compile to evaluate the CHECK expression
        #[allow(clippy::let_unit_value)]
        let _check = Self::CHECK;

        let values = patch.references();
        let values: Vec<_> = values.iter().map(Value::as_sql).collect();

        let mut ctx = QueryContext::new();
        let decoder = self.selector.select(&mut ctx);
        let returning = ctx
            .get_returning()
            .expect("Should have been checked in set_select");

        let row = database::insert_returning(
            self.executor,
            P::Model::TABLE,
            P::COLUMNS,
            &values,
            &returning,
        )
        .await?;
        decoder.by_index(&row)
    }

    /// Insert a bulk of patches into the db
    ///
    /// # Argument
    /// This method accepts anything which can be used to iterate
    /// over instances or references of your [`Patch`].
    ///
    /// **Examples**: (where `P` is your patch)
    /// - `Vec<P>`
    /// - `&[P]`
    /// - A [`map`](Iterator::map) iterator yielding `P` or `&P`
    pub async fn bulk<'p, I, P>(self, patches: I) -> Result<Vec<S::Result>, Error>
    where
        I: IntoIterator,
        I::Item: IntoPatchCow<'p, Patch = P>,
        P: Patch<Model = M>,
    {
        // it is intentional to force the compile to evaluate the CHECK expression
        #[allow(clippy::let_unit_value)]
        let _check = Self::CHECK;

        let mut values: Vec<Value<'p>> = Vec::new();
        for patch in patches {
            match patch.into_patch_cow() {
                PatchCow::Borrowed(patch) => patch.push_references(&mut values),
                PatchCow::Owned(patch) => patch.push_values(&mut values),
            }
        }

        let values: Vec<_> = values.iter().map(Value::as_sql).collect();
        let values_slices: Vec<_> = values.chunks(P::COLUMNS.len()).collect();

        let mut ctx = QueryContext::new();
        let decoder = self.selector.select(&mut ctx);
        let returning = ctx
            .get_returning()
            .expect("Should have been checked in set_select");

        let rows = database::insert_bulk_returning(
            self.executor,
            M::TABLE,
            P::COLUMNS,
            &values_slices,
            &returning,
        )
        .await?;
        rows.iter().map(|row| decoder.by_index(row)).collect()
    }
}

/// Variation of [`InsertBuilder`] which performs an insert without returning anything
#[must_use]
pub struct InsertReturningNothing<E, M> {
    executor: E,
    model: PhantomData<M>,
}
impl<'ex, E, M> InsertReturningNothing<E, M>
where
    E: Executor<'ex>,
    M: Model,
{
    /// See [`InsertBuilder::single`]
    pub async fn single<P: Patch<Model = M>>(self, patch: &P) -> Result<(), Error> {
        let values = patch.references();
        let values: Vec<_> = values.iter().map(Value::as_sql).collect();
        let inserting = P::COLUMNS;

        database::insert(self.executor, M::TABLE, inserting, &values).await
    }

    /// See [`InsertBuilder::bulk`]
    pub async fn bulk<'p, I, P>(self, patches: I) -> Result<(), Error>
    where
        I: IntoIterator,
        I::Item: IntoPatchCow<'p, Patch = P>,
        P: Patch<Model = M>,
    {
        let mut values: Vec<Value<'p>> = Vec::new();
        for patch in patches {
            match patch.into_patch_cow() {
                PatchCow::Borrowed(patch) => patch.push_references(&mut values),
                PatchCow::Owned(patch) => patch.push_values(&mut values),
            }
        }

        let values: Vec<_> = values.iter().map(Value::as_sql).collect();
        let values_slices: Vec<_> = values.chunks(P::COLUMNS.len()).collect();
        let inserting = P::COLUMNS;

        database::insert_bulk(self.executor, M::TABLE, inserting, &values_slices).await
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
            <<$patch as $crate::model::Patch>::Model as $crate::model::Model>::permissions()
                .insert_permission(),
        )
    };
}
