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
//! It the proceeds to implement then [`RawField`] trait on this empty struct.
//! Therefore, [`RawField`] encapsulates all information the macro can gather.
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
//! impl RawField for __User_id {
//!     type RawType = i32;
//!     const NAME: &'static str = "id";
//!     ...
//! }
//! ```
//!
//! ---
//!
//! From there on, further traits are implemented using generic `impl`s defined in this module.
//! These implementations branch depending on the field's type.
//!
//! **This hits a limitation in rust:**
//! We need to provide different generic implementations for the same traits ([`AbstractField`] and [`Field`]).
//! rust enforces implementations to don't overlap.
//! To achieve this a [`FieldKind`] is introduced.
//! Each [`FieldType`] (a type usable as a field) is of exactly one such kind.
//! Using this kinds as constraints for the generic [`RawField]'s type,
//! should make these implementation branches mutually exclusive.
//! However rust doesn't quite understand this, which is due to an old bug (stated by some online sources).
//!
//! As a workaround all traits after [`RawField`] carry a generic [`FieldKind`] which defaults to `<Self as RawField>::Kind`.
//! This way
//! - The traits (for example `Field<kind::AsDbType` and `Field<kind::ForeignModel>`)
//! are treated as different traits, as far as the impl overlap is concerned.
//! - You can write `F: Field` in constraint without having to state the generic every time.
//!
//! *(Thank you a lot to whomever's blog post I read to figure all this out.
//! I'm sorry, I couldn't find you anymore to credit you properly.)*
//!
//! ---
//!
//! **The concrete branches are experimental and might change any time!**
//!
//! The [`Field`] implementation does further processing of [`RawField`].
//! For example it merges the annotations set by the user with annotations implied by the raw type
//! and runs a linter shared with `rorm-cli` on them.

use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use rorm_db::row::DecodeOwned;
use rorm_db::{Error, Row};
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::internal::hmr::{AsImr, Source};
use crate::internal::relation_path::{Path, PathImpl, PathStep, ResolvedRelatedField};
use crate::model::{ConstNew, Model};
use crate::{const_concat, const_panic, sealed};

pub mod as_db_type;
pub mod datetime;
pub mod foreign_model;

use as_db_type::AsDbType;

/// Marker trait for various kinds of fields
pub trait FieldKind {
    sealed!(trait);
}
/// Namespace for the different [`FieldKind`] impls.
pub mod kind {
    use super::FieldKind;
    use crate::sealed;

    /// Marker for some field which is a [`ForeignModel`](crate::fields::ForeignModelByField)
    pub struct ForeignModel;
    /// Marker for some field which is a [`BackRef`](crate::fields::BackRef)
    pub struct BackRef;
    /// Marker for some field which is an [`AsDbType`](crate::internal::field::as_db_type::AsDbType)
    pub struct AsDbType;
    /// Marker for some field which is an [`DateTime<FixedOffset>`](chrono::DateTime)
    pub struct DateTime;

    impl FieldKind for ForeignModel {
        sealed!(impl);
    }
    impl FieldKind for BackRef {
        sealed!(impl);
    }
    impl FieldKind for AsDbType {
        sealed!(impl);
    }
    impl FieldKind for DateTime {
        sealed!(impl);
    }
}

/// The type of field allowed on models
pub trait FieldType {
    /// The kind of field this type declares
    type Kind: FieldKind;

    /// Array with length specific to the field type
    type Columns<'a>: IntoIterator<Item = Value<'a>>;

    /// Construct an array of [`Value`] representing `self` in the database via ownership
    fn into_values(self) -> Self::Columns<'static>;

    /// Construct an array of [`Value`] representing `self` in the database via borrowing
    fn as_values(&self) -> Self::Columns<'_>;
}

/// This trait is implemented by the `#[derive(Model)]` macro on unique unit struct for each of a model's fields.
///
/// It contains all the information a model's author provides on the field.
///
/// This trait itself doesn't do much, but it forms the basis to implement the other traits.
pub trait RawField: 'static + Copy {
    /// The field's kind which is determined by its [type](RawField::Type)
    type Kind: FieldKind;

    /// The type stored in the model's field
    type Type: FieldType<Kind = Self::Kind>;

    /// The model this field is part of
    type Model: Model;

    /// This field's position in the model
    const INDEX: usize;

    /// A db safe name of this field
    const NAME: &'static str;

    /// List of annotations which were set by the user
    const EXPLICIT_ANNOTATIONS: Annotations;

    /// Optional definition of the location of field in the source code
    const SOURCE: Option<Source>;

    /// Create a new instance
    ///
    /// Since `Self` is always a zero sized type, this is a noop.
    /// It exists to enable accessing field method through [`FieldProxy`] without having to forward every one.
    fn new() -> Self;
}

/// A [`RawField`] which represents a single column in the database
pub trait Field<K: FieldKind = <Self as RawField>::Kind>: RawField {
    sealed!(trait);

    /// The data type as which this field is stored in the db
    type DbType: DbType;

    /// List of the actual annotations
    const ANNOTATIONS: Annotations;

    /// Entry point for compile time checks on a single field
    ///
    /// It is "used" in [`FieldProxy::new`] to force the compiler to evaluate it.
    const CHECK: usize = {
        // Are required annotations set?
        let mut required = Self::DbType::REQUIRED;
        while let [head, tail @ ..] = required {
            required = tail;
            if !Self::ANNOTATIONS.is_set(head) {
                const_panic!(&[
                    Self::Model::TABLE,
                    ".",
                    Self::NAME,
                    " requires the annotation `",
                    head.as_str(),
                    "` but it's missing",
                ]);
            }
        }

        // Run the annotations lint shared with rorm-cli
        let annotations = Self::ANNOTATIONS.as_lint();
        if let Err(err) = annotations.check() {
            const_panic!(&[
                Self::Model::TABLE,
                ".",
                Self::NAME,
                " has invalid annotations: ",
                err,
            ]);
        }

        Self::INDEX
    };

    /// A type which can be retrieved from the db and then converted into [`Self::Type`](RawField::Type).
    type Primitive: DecodeOwned;

    /// Convert the associated primitive type into [`Self::Type`](RawField::Type).
    fn from_primitive(self, primitive: Self::Primitive) -> Self::Type;

    /// The [`Field`] contract requires [`RawField::into_values`] to only return a single value
    ///
    /// This method is a shorthand to access this value
    fn into_value(self, value: Self::Type) -> Value<'static> {
        let Some(value) = extract_value(value.into_values()) else {
            unreachable!("A `Field` may only contain one column")
        };
        value
    }

    /// The [`Field`] contract requires [`RawField::as_values`] to only return a single value
    ///
    /// This method is a shorthand to access this value
    fn as_value(self, value: &Self::Type) -> Value {
        let Some(value) = extract_value(value.as_values()) else {
            unreachable!("A `Field` may only contain one column")
        };
        value
    }
}

impl<T: AsDbType, F: RawField<Type = T, Kind = kind::AsDbType>> Field<kind::AsDbType> for F {
    sealed!(impl);

    type DbType = <T as AsDbType>::DbType;

    const ANNOTATIONS: Annotations = {
        if let Some(implicit) = Self::Type::IMPLICIT {
            match Self::EXPLICIT_ANNOTATIONS.merge(implicit) {
                Ok(annotations) => annotations,
                Err(duplicate) => {
                    const_panic!(&[
                        "The annotation ",
                        duplicate,
                        " on ",
                        Self::Model::TABLE,
                        ".",
                        Self::NAME,
                        " is implied by its type and can't be set explicitly",
                    ]);
                }
            }
        } else {
            Self::EXPLICIT_ANNOTATIONS
        }
    };

    type Primitive = T::Primitive;

    fn from_primitive(self, primitive: Self::Primitive) -> Self::Type {
        T::from_primitive(primitive)
    }
}

/// A common interface unifying the fields of various kinds.
pub trait AbstractField<K: FieldKind = <Self as RawField>::Kind>: RawField {
    sealed!(trait);

    /// Add the field to its model's intermediate model representation
    ///
    /// - [`kind::BackRef`] fields don't add anything
    /// - [`Field`] fields add their database column
    /// - there are plans to add fields which might map to more than one database column.
    fn push_imr(self, imr: &mut Vec<imr::Field>);

    /// Get an instance of the field's type from a row using the field's name
    fn get_by_name(self, row: &Row) -> Result<Self::Type, Error>;

    /// Get an instance of the field's type from a row by its position in the SELECT query.
    ///
    /// # Multi-column fields
    ///
    /// Since this type of field has more than one column, it can't just access the one column at the one index.
    /// However the row contains the field's columns in the order defined by [`Self::COLUMNS`].
    /// So use offsets to access them.
    ///
    /// ```ignore
    /// impl AbstractField for F {
    ///     const COLUMNS: &'static [&'static str] = &["foo", "bar", "baz"];
    ///
    ///     fn get_by_index(row: &Row, index: usize) -> Result<Self::Type, Error> {
    ///         let foo = row.get(index + 0)?;
    ///         let bar = row.get(index + 1)?;
    ///         let baz = row.get(index + 2)?;
    ///         Ok(todo!())
    ///     }
    /// }
    /// ```
    fn get_by_index(self, row: &Row, index: usize) -> Result<Self::Type, Error>;

    /// The columns' names which store this field
    const COLUMNS: &'static [&'static str] = &[];

    /// The list of annotations, if this field is relevant to the database.
    const DB_ANNOTATIONS: Option<Annotations> = None;
}
macro_rules! impl_abstract_from_field {
    ($kind:ty) => {
        impl<F: Field<$kind>> AbstractField<$kind> for F {
            sealed!(impl);

            fn push_imr(self, imr: &mut Vec<imr::Field>) {
                imr.push(imr::Field {
                    name: F::NAME.to_string(),
                    db_type: F::DbType::IMR,
                    annotations: F::ANNOTATIONS.as_imr(),
                    source_defined_at: F::SOURCE.map(|s| s.as_imr()),
                });
            }

            fn get_by_name(self, row: &Row) -> Result<Self::Type, Error> {
                Ok(<Self as RawField>::new().from_primitive(row.get(F::NAME)?))
            }

            fn get_by_index(self, row: &Row, index: usize) -> Result<Self::Type, Error> {
                Ok(<Self as RawField>::new().from_primitive(row.get(index)?))
            }

            const COLUMNS: &'static [&'static str] = &[F::NAME];

            const DB_ANNOTATIONS: Option<Annotations> = {
                // "Use" the CHECK constant to force the compiler to evaluate it.
                let _check: usize = F::CHECK;
                Some(F::ANNOTATIONS)
            };
        }

        impl<P: Path, F: Field<$kind>> AliasedField<P, $kind> for F {
            const COLUMNS: &'static [&'static str] = &[const_concat!(&[P::ALIAS, "__", F::NAME])];

            fn get_by_alias(row: &Row) -> Result<Self::Type, Error> {
                Ok(<Self as RawField>::new()
                    .from_primitive(row.get(<Self as AliasedField<P, $kind>>::COLUMNS[0])?))
            }
        }
    };
}
impl_abstract_from_field!(kind::AsDbType);
impl_abstract_from_field!(kind::ForeignModel);

/// Helper trait to work with fields which are accessed through an alias.
pub trait AliasedField<P: Path, K: FieldKind = <Self as RawField>::Kind>: RawField {
    /// The field's columns prefixed with `P`'s alias
    const COLUMNS: &'static [&'static str];

    /// Retrieve the field's value using its alias as index
    fn get_by_alias(row: &Row) -> Result<Self::Type, Error>;
}

/// This struct acts as a proxy exposing type level information from the [`RawField`] trait on the value level.
///
/// On top of that it can be used to keep track of the "path" this field is accessed through, when dealing with relations.
///
/// ## Type as Value
/// In other words [`FieldProxy`] allows access to things like [`RawField::NAME`] without access to the concrete field type.
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
impl<F: RawField, P> FieldProxy<F, P> {
    /// Create a new instance
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    /// Get the field's position in the Model
    pub const fn index(_field: Self) -> usize {
        F::INDEX
    }
}
impl<F: AbstractField, P> FieldProxy<F, P> {
    /// Get the names of the columns which store the field
    pub const fn columns(_field: Self) -> &'static [&'static str] {
        F::COLUMNS
    }

    /// Get the underlying field to call its methods
    pub fn field(&self) -> F {
        F::new()
    }
}
impl<Field, Path> Clone for FieldProxy<Field, Path> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<Field, Path> Copy for FieldProxy<Field, Path> {}

/// A field whose proxy should implement [`Deref`](std::ops::Deref) to some collection of fields.
///
/// Depending on the field, this collection might differ in meaning
/// - For [`BackRef`](crate::fields::BackRef) and [`ForeignModel`](crate::fields::ForeignModelByField),
///   its their related model's fields
/// - For multi-column fields, its their "contained" fields
pub trait ContainerField<P: Path, K: FieldKind = <Self as RawField>::Kind>: RawField {
    /// Struct of contained fields
    type Target: ConstNew;
}

impl<F: RawField, P: Path> std::ops::Deref for FieldProxy<F, P>
where
    F: ContainerField<P>,
{
    type Target = F::Target;

    fn deref(&self) -> &'static Self::Target {
        ConstNew::REF
    }
}

impl<F, P> ContainerField<P, kind::ForeignModel> for F
where
    P: Path,
    F: RawField<Kind = kind::ForeignModel>,
    PathStep<F, P>: PathImpl<F::Type>,
{
    type Target =
        <<ResolvedRelatedField<F, P> as RawField>::Model as Model>::Fields<PathStep<F, P>>;
}

impl<F, P> ContainerField<P, kind::BackRef> for F
where
    P: Path,
    F: RawField<Kind = kind::BackRef>,
    PathStep<F, P>: PathImpl<F::Type>,
{
    type Target =
        <<ResolvedRelatedField<F, P> as RawField>::Model as Model>::Fields<PathStep<F, P>>;
}

/// Check if a container of [`Values`] contains a single value and returns it.
///
/// If it doesn't, returns nothing
fn extract_value<'a>(values: impl IntoIterator<Item = Value<'a>>) -> Option<Value<'a>> {
    let mut values = values.into_iter();
    let value = values.next()?;
    if values.next().is_none() {
        Some(value)
    } else {
        None
    }
}
