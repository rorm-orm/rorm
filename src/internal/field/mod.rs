//! The various field traits and their proxy.
//!
//! # Introduction
//! Rorm's main entry point is the [`Model`] trait and its derive macro.
//! It takes a struct and generates the code to represent this struct as a database table.
//! To do so each of the struct's fields need to be represented in some way.
//!
//! For each field the derive macro declares a unit struct (i.e. an empty struct) to represent it.
//! This empty struct is then "populated" with the field's information using various traits defined in this module.
//!
//! # Trait Implementation Flow
//! As stated in the introduction, the derive macro generates an unit struct per field.
//! It the proceeds to implement then [`Field`] trait on this empty struct.
//! Therefore, [`Field`] encapsulates all information the macro can gather.
//! This includes:
//! - the name (a db safe version of it, to be precise)
//! - its "raw type" ("raw" because the macro can't make any deductions about the type)
//! - the various annotations inside a `#[rorm(...)]` attribute
//!
//! #### Small illustration
//! ```text
//! #[derive(Model)]
//! struct User {
//!     id: i32,
//!     ...
//! }
//! ```
//! will produce something like
//! ```text
//! struct __User_id;
//! impl Field for __User_id {
//!     type RawType = i32;
//!     const NAME: &'static str = "id";
//!     ...
//! }
//! ```
//!
//! From there the various methods and associated type from [`FieldType`] take over.
//! TODO more docs

use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::conditions::Value;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::Source;
use crate::internal::relation_path::{Path, PathImpl, PathStep};
use crate::model::{ConstNew, GetField, Model};

pub mod access;
pub mod as_db_type;
pub mod decoder;
pub mod foreign_model;
pub mod modifier;

use crate::fields::traits::FieldType;
use crate::fields::types::{BackRef, ForeignModelByField};
use crate::internal::array_utils::IntoArray;
use crate::internal::const_concat::ConstString;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::foreign_model::{ForeignModelField, ForeignModelTrait};
use crate::internal::field::modifier::{AnnotationsModifier, CheckModifier, ColumnsFromName};

/// This trait is implemented by the `#[derive(Model)]` macro on unique unit struct for each of a model's fields.
///
/// It contains all the information a model's author provides on the field.
///
/// This trait itself doesn't do much, but it forms the basis to implement the other traits.
pub trait Field: 'static + Copy {
    /// The type stored in the model's field
    type Type: FieldType;

    /// The model this field is part of
    type Model: Model;

    /// This field's position in the model
    const INDEX: usize;

    /// A db safe name of this field
    const NAME: &'static str;

    /// List of annotations which were set by the user
    const EXPLICIT_ANNOTATIONS: Annotations;

    /// List of annotations which are passed to db, if this field is a single column
    const EFFECTIVE_ANNOTATIONS: Option<Annotations> =
        { <Self::Type as FieldType>::AnnotationsModifier::<Self>::MODIFIED };

    /// Compile time check and it error message
    ///
    /// The const is accessed and reported in the `#[derive(Model)]`.
    const CHECK: Result<(), ConstString<1024>> =
        <Self::Type as FieldType>::CheckModifier::<Self>::RESULT;

    /// Optional definition of the location of field in the source code
    const SOURCE: Option<Source>;

    /// Create a new instance
    ///
    /// Since `Self` is always a zero sized type, this is a noop.
    /// It exists to enable accessing field method through [`FieldProxy`] without having to forward every one.
    fn new() -> Self;
}

/// A field which is stored in db via a single column
pub trait SingleColumnField: Field {
    /// Borrow an instance of the field's type as a [`Value`]
    fn type_as_value(field: &Self::Type) -> Value;

    /// Convert an instance of the field's type into a static [`Value`]
    fn type_into_value(field: Self::Type) -> Value<'static>;
}
impl<F> SingleColumnField for F
where
    F: Field,
    for<'a> <F::Type as FieldType>::Columns<Value<'a>>: IntoArray<1>,
{
    fn type_as_value(field: &Self::Type) -> Value {
        let [value] = field.as_values().into_array();
        value
    }

    fn type_into_value(field: Self::Type) -> Value<'static> {
        let [value] = field.into_values().into_array();
        value
    }
}

/// This struct acts as a proxy exposing type level information from the [`Field`] trait on the value level.
///
/// On top of that it can be used to keep track of the "path" this field is accessed through, when dealing with relations.
///
/// ## Type as Value
/// In other words [`FieldProxy`] allows access to things like [`Field::NAME`] without access to the concrete field type.
///
/// Pseudo code for illustration:
/// ```skip
/// // The following is a rough sketch of what the #[derive(Model)] will do:
/// pub struct Id;
/// impl Field for Id {
///     ...
/// }
///
/// pub struct Fields {
///     pub id: FieldProxy<Id>,
///     ...
/// }
///
/// pub struct User {
///     pub id: i32,
/// }
/// impl Model for User {
///     type Fields = Fields;
///     const FIELDS: Self::Fields = Fields {
///         id: Id,
///         ...
///     }
/// }
///
/// // To access Id::NAME from user code, we can't use the Field trait itself,
/// // because the type Id is not really accessible. (It's been generated from a macro.)
/// // Also User::FIELDS or User::F should have more of a struct like syntax.
/// //
/// // So, the Fields struct holds FieldProxy<Id> instead of Id, which implements simple methods
/// // forwarding varies data and behaviors from Id:
///
/// Id::NAME ~ User::F.id.name()
/// Id::Index ~ User::F.id.index()
/// Id::Type::from_primitive ~ User::F.id.convert_primitive
/// ```
pub struct FieldProxy<Field, Path>(PhantomData<ManuallyDrop<(Field, Path)>>);

// SAFETY:
// struct contains no data
unsafe impl<F, P> Send for FieldProxy<F, P> {}

impl<F: Field, P> FieldProxy<F, P> {
    /// Create a new instance
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    /// Get the field's position in the Model
    pub const fn index(_field: Self) -> usize {
        F::INDEX
    }

    /// Change the path
    pub const fn through<NewP>(self) -> FieldProxy<F, NewP> {
        FieldProxy::new()
    }
}
impl<F: Field, P> FieldProxy<F, P> {
    /// Get the names of the columns which store the field
    pub const fn columns(_field: Self) -> <F::Type as FieldType>::Columns<&'static str> {
        <F::Type as FieldType>::ColumnsFromName::<F>::COLUMNS
    }

    /// Get the underlying field to call its methods
    pub fn field(&self) -> F {
        F::new()
    }
}
impl<Field, Path> Clone for FieldProxy<Field, Path> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Field, Path> Copy for FieldProxy<Field, Path> {}

/// A field whose proxy should implement [`Deref`](std::ops::Deref) to some collection of fields.
///
/// Depending on the field, this collection might differ in meaning
/// - For [`BackRef`](crate::fields::types::BackRef) and [`ForeignModel`](crate::fields::types::ForeignModelByField),
///   its their related model's fields
/// - For multi-column fields, its their "contained" fields
pub trait ContainerField<T: FieldType, P: Path>: Field<Type = T> {
    /// Struct of contained fields
    type Target: ConstNew;
}

impl<T: FieldType, F: Field<Type = T>, P: Path> std::ops::Deref for FieldProxy<F, P>
where
    F: ContainerField<T, P>,
{
    type Target = F::Target;

    fn deref(&self) -> &'static Self::Target {
        ConstNew::REF
    }
}

impl<FMF, BF, P> ContainerField<BackRef<FMF>, P> for BF
where
    FMF: ForeignModelField,
    BF: Field<Type = BackRef<FMF>>,
    P: Path,
    PathStep<BF, P>: PathImpl<BackRef<FMF>>,
{
    // type Target = <<ResolvedRelatedField<BF, P> as Field>::Model as Model>::Fields<PathStep<BF, P>>;
    type Target = <FMF::Model as Model>::Fields<PathStep<BF, P>>;
}

impl<FF, FMF, P> ContainerField<ForeignModelByField<FF>, P> for FMF
where
    // bound in `impl FieldType for ForeignModelByField<FF>`
    ForeignModelByField<FF>: ForeignModelTrait,
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true

    FMF: Field<Type = ForeignModelByField<FF>>,
    P: Path,
    PathStep<FMF, P>: PathImpl<ForeignModelByField<FF>>,
{
    type Target = <FF::Model as Model>::Fields<PathStep<FMF, P>>;
}

impl<FF, FMF, P> ContainerField<Option<ForeignModelByField<FF>>, P> for FMF
where
    // bound in `impl FieldType for ForeignModelByField<FF>`
    Option<ForeignModelByField<FF>>: ForeignModelTrait,
    FF: SingleColumnField,
    FF::Type: AsDbType,
    FF::Model: GetField<FF>, // always true
    Option<FF::Type>: AsDbType,

    FMF: Field<Type = Option<ForeignModelByField<FF>>>,
    P: Path,
    PathStep<FMF, P>: PathImpl<Option<ForeignModelByField<FF>>>,
{
    type Target = <FF::Model as Model>::Fields<PathStep<FMF, P>>;
}
