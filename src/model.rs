//! This module holds traits and structs for working with models

use std::marker::PhantomData;

use rorm_declaration::imr;

use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::crud::decoder::Decoder;
use crate::crud::selector::Selector;
use crate::internal::field::{Field, FieldProxy, SingleColumnField};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;

/// Trait implemented on Patches i.e. a subset of a model's fields.
///
/// Implemented by [`derive(Patch)`] as well as [`derive(Model)`].
pub trait Patch: Sized + 'static {
    /// The model this patch is for
    type Model: Model;

    /// [`Decoder`] returned by [`Patch::select`] which decodes this patch from a row
    type Decoder: Decoder<Result = Self>;

    /// Constructs a [`Self::Decoder`] and configures a [`QueryContext`] to query the required columns
    ///
    /// (Cmp [`Selector`])
    fn select<P: Path>(ctx: &mut QueryContext) -> Self::Decoder;

    /// List of columns i.e. fields this patch contains
    const COLUMNS: &'static [&'static str];

    /// Create a [`Vec`] moving the patch's condition values
    fn values(self) -> Vec<Value<'static>> {
        let mut values = Vec::with_capacity(Self::COLUMNS.len());
        self.push_values(&mut values);
        values
    }

    /// Push the patch's condition values onto a [`Vec`]
    fn push_values(self, values: &mut Vec<Value>);

    /// Create a [`Vec`] borrowing the patch's condition values
    fn references(&self) -> Vec<Value> {
        let mut values = Vec::with_capacity(Self::COLUMNS.len());
        self.push_references(&mut values);
        values
    }

    /// Push the patch's condition values onto a [`Vec`]
    fn push_references<'a>(&'a self, values: &mut Vec<Value<'a>>);
}

/// [`Selector`] selecting a [`Patch`] through its [`Patch::select`] method
pub struct PatchSelector<Ptch: Patch, Pth = <Ptch as Patch>::Model>(PhantomData<(Ptch, Pth)>);

impl<Ptch: Patch, Pth> PatchSelector<Ptch, Pth> {
    /// construct a new instance
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Ptch: Patch, Pth: Path> Default for PatchSelector<Ptch, Pth> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Ptch: Patch, Pth: Path> Selector for PatchSelector<Ptch, Pth> {
    type Result = Ptch;
    type Model = Pth::Origin;
    type Decoder = Ptch::Decoder;
    const INSERT_COMPATIBLE: bool = Pth::IS_ORIGIN;

    fn select(self, ctx: &mut QueryContext) -> Self::Decoder {
        Pth::add_to_context(ctx);
        Ptch::select::<Pth>(ctx)
    }
}

/// The [Condition](crate::conditions::Condition) type returned by [Identifiable::as_condition]
pub type PatchAsCondition<'a, P> = Binary<
    Column<FieldProxy<<<P as Patch>::Model as Model>::Primary, <P as Patch>::Model>>,
    Value<'a>,
>;

/// Trait implementing most database interactions for a struct.
///
/// It should only ever be generated using [`derive(Model)`](rorm_macro::Model).
pub trait Model: Patch<Model = Self> {
    /// The primary key
    type Primary: Field<Model = Self> + SingleColumnField;

    /// A struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    ///
    /// The struct is constructed once in the [`Model::FIELDS`] constant.
    type Fields<P: Path>: ConstNew;

    /// A constant struct which "maps" field identifiers their descriptions (i.e. [`Field<T>`](crate::internal::field::Field)).
    const FIELDS: Self::Fields<Self>;

    /// Shorthand version of [`FIELDS`]
    ///
    /// [`FIELDS`]: Model::FIELDS
    const F: Self::Fields<Self>;

    /// The model's table name
    const TABLE: &'static str;

    /// Returns the model's intermediate representation
    ///
    /// As library user you probably won't need this. You might want to look at [`write_models`].
    ///
    /// [`write_models`]: crate::write_models
    fn get_imr() -> imr::Model;

    /// Returns a zero sized type which constructs the CRUD permission tokens.
    ///
    /// It's methods `fn ..._permission(&self) -> ...Permission` are either locked behind
    /// a trait bound `Model<...Permission = Unrestricted>`
    /// or the equivalent methods on [`Model::Permissions`] which are locked behind rust visibilities.
    fn permissions() -> ModelPermissions<Self> {
        ModelPermissions(Default::default())
    }

    /// Zero sized type which constructs the permission tokens.
    ///
    /// Use [`Model::permissions`]
    type Permissions: Default + Send + Sync + Sized + 'static;

    /// Zero sized token which grants the permission to use [`insert`]
    type InsertPermission: Send + Sync + Sized + 'static;

    /// Zero sized token which grants the permission to use [`query`]
    type QueryPermission: Send + Sync + Sized + 'static;

    /// Zero sized token which grants the permission to use [`update`]
    type UpdatePermission: Send + Sync + Sized + 'static;

    /// Zero sized token which grants the permission to use [`delete`]
    type DeletePermission: Send + Sync + Sized + 'static;
}

/// Zero sized type which constructs the CRUD permission tokens for a [`Model`].
///
/// It's methods `fn ..._permission(&self) -> ...Permission` are either locked behind
/// a trait bound `Model<...Permission = Unrestricted>`
/// or the equivalent methods on [`Model::Permissions`] which are locked behind rust visibilities.
pub struct ModelPermissions<M: Model>(M::Permissions);
impl<M: Model<InsertPermission = Unrestricted>> ModelPermissions<M> {
    /// Get permission to use [`insert!`](crate::insert).
    ///
    /// This method is either restricted by a visibility set by the `#[derive(Model)]`
    /// or the trait bound `M: Model<InsertPermission = Unrestricted>`
    /// if the macro didn't specify any visibility.
    pub fn insert_permission(&self) -> M::InsertPermission {
        Unrestricted(PhantomData)
    }
}
impl<M: Model<QueryPermission = Unrestricted>> ModelPermissions<M> {
    /// Get permission to use [`query!`](crate::query).
    ///
    /// This method is either restricted by a visibility set by the `#[derive(Model)]`
    /// or the trait bound `M: Model<QueryPermission = Unrestricted>`
    /// if the macro didn't specify any visibility.
    pub fn query_permission(&self) -> M::QueryPermission {
        Unrestricted(PhantomData)
    }
}
impl<M: Model<UpdatePermission = Unrestricted>> ModelPermissions<M> {
    /// Get permission to use [`update!`](crate::update).
    ///
    /// This method is either restricted by a visibility set by the `#[derive(Model)]`
    /// or the trait bound `M: Model<UpdatePermission = Unrestricted>`
    /// if the macro didn't specify any visibility.
    pub fn update_permission(&self) -> M::UpdatePermission {
        Unrestricted(PhantomData)
    }
}
impl<M: Model<DeletePermission = Unrestricted>> ModelPermissions<M> {
    /// Get permission to use [`delete!`](crate::delete).
    ///
    /// This method is either restricted by a visibility set by the `#[derive(Model)]`
    /// or the trait bound `M: Model<DeletePermission = Unrestricted>`
    /// if the macro didn't specify any visibility.
    pub fn delete_permission(&self) -> M::DeletePermission {
        Unrestricted(PhantomData)
    }
}
impl<M: Model> std::ops::Deref for ModelPermissions<M> {
    type Target = M::Permissions;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Zero sized token which can be constructed by everyone
pub struct Unrestricted(pub PhantomData<()>);

/// Expose a models' fields on the type level using indexes
pub trait FieldByIndex<const INDEX: usize>: Model {
    /// The model's field at `INDEX`
    type Field: Field<Model = Self>;
}

/// Generic access to a patch's fields
///
/// This enables generic code to check if a patch contains a certain field
/// (for example the model's primary key, see [Identifiable])
/// and gain access to it.
pub trait GetField<F: Field>: Patch {
    /// Take the field by ownership
    fn get_field(self) -> F::Type;

    /// Borrow the field
    fn borrow_field(&self) -> &F::Type;

    /// Borrow the field mutably
    fn borrow_field_mut(&mut self) -> &mut F::Type;
}

/// Update a model's field based on the model's primary key
///
/// This trait is similar to [`GetField::borrow_field_mut`].
/// But [`GetField::borrow_field_mut`] only allows access to one field at a time,
/// because the method hides the fact, that the mutual borrow only applies to a single field.
/// This trait provides a solution to this problem, for a common scenario:
/// The need for an additional immutable borrow to the primary key.
pub trait UpdateField<F: Field<Model = Self>>: Model {
    /// Update a model's field based on the model's primary key
    fn update_field<'m, T>(
        &'m mut self,
        update: impl FnOnce(&'m <<Self as Model>::Primary as Field>::Type, &'m mut F::Type) -> T,
    ) -> T;
}

/// A patch which contains its model's primary key.
pub trait Identifiable: Patch {
    /// Get a reference to the primary key
    fn get_primary_key(&self) -> &<<Self::Model as Model>::Primary as Field>::Type;

    /// Build a [Condition](crate::conditions::Condition)
    /// which only applies to this instance by comparing the primary key.
    fn as_condition(&self) -> PatchAsCondition<Self> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(FieldProxy::new()),
            snd_arg: <Self::Model as Model>::Primary::type_as_value(self.get_primary_key()),
        }
    }
}

impl<M: Model, P: Patch<Model = M> + GetField<M::Primary>> Identifiable for P {
    fn get_primary_key(&self) -> &<M::Primary as Field>::Type {
        <Self as GetField<M::Primary>>::borrow_field(self)
    }
}

/// exposes a `NEW` constant, which act like [Default::default] but constant.
///
/// It's workaround for not having const methods in traits
pub trait ConstNew: 'static {
    /// A new or default instance
    const NEW: Self;

    /// A static reference to an default instance
    ///
    /// Sadly writing `const REF: &'static Self = &Self::NEW;` doesn't work for all `Self`.
    /// Rust doesn't allow references to types with interior mutability to be stored in constants.
    /// Since this can't be enforced by generic, `ConstNew` impls have to write this line themselves.
    const REF: &'static Self;
}
