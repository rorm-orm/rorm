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

use rorm_db::sql::value::NullType;
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::fields::proxy::FieldProxy;
use crate::fields::proxy::FieldProxyImpl;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::{AsImr, Source};
use crate::internal::relation_path::{Path, PathField};
use crate::model::{ConstNew, Model};

pub mod decoder;
pub mod fake_field;
pub mod foreign_model;
pub mod multi_column;

use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::column_name::ColumnName;
use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::const_concat::ConstString;

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
    const NAME: ColumnName;

    /// List of annotations which were set by the user
    const EXPLICIT_ANNOTATIONS: Annotations;

    /// List of annotations which are passed to db
    const EFFECTIVE_ANNOTATIONS: FieldColumns<Self::Type, Annotations> =
        <<<Self::Type as FieldType>::GetAnnotations as ConstFn<_, _>>::Body<(
            contains::ExplicitAnnotations<Self>,
        )> as Contains<_>>::ITEM;

    /// List of names which are passed to db
    const EFFECTIVE_NAMES: FieldColumns<Self::Type, ColumnName> =
        <<<Self::Type as FieldType>::GetNames as ConstFn<_, _>>::Body<(contains::Name<Self>,)> as Contains<_>>::ITEM;

    /// Location of the field in the source code
    const SOURCE: Source;

    /// Create a new instance
    ///
    /// Since `Self` is always a zero sized type, this is a noop.
    /// It exists to enable accessing field method through [`FieldProxy`] without having to forward every one.
    fn new() -> Self;
}

/// Pushes a [`Field`]'s columns as [`imr`] onto a vector.
///
/// This function is called by the `#[derive(Model)]` macro to gather a list of all vectors.
pub fn push_imr<F: Field>(imr: &mut Vec<imr::Field>) {
    let names = F::EFFECTIVE_NAMES;
    let db_types = F::Type::NULL;
    let annotations = F::EFFECTIVE_ANNOTATIONS;
    let source_defined_at = F::SOURCE.as_imr();

    for ((name, annotations), null_type) in names
        .into_iter()
        .zip(annotations.into_iter())
        .zip(db_types.into_iter())
    {
        imr.push(imr::Field {
            name: name.as_str().to_string(),
            db_type: match null_type {
                NullType::String => imr::DbType::VarChar,
                NullType::Choice => imr::DbType::Choices,
                NullType::I64 => imr::DbType::Int64,
                NullType::I32 => imr::DbType::Int32,
                NullType::I16 => imr::DbType::Int16,
                NullType::Bool => imr::DbType::Boolean,
                NullType::F64 => imr::DbType::Double,
                NullType::F32 => imr::DbType::Float,
                NullType::Binary => imr::DbType::Binary,
                NullType::ChronoNaiveTime => imr::DbType::Time,
                NullType::ChronoNaiveDate => imr::DbType::Date,
                NullType::ChronoNaiveDateTime => imr::DbType::DateTime,
                NullType::ChronoDateTime => imr::DbType::DateTime,
                NullType::TimeDate => imr::DbType::Date,
                NullType::TimeTime => imr::DbType::Time,
                NullType::TimeOffsetDateTime => imr::DbType::DateTime,
                NullType::TimePrimitiveDateTime => imr::DbType::DateTime,
                NullType::Uuid => imr::DbType::Uuid,
                NullType::UuidHyphenated => imr::DbType::Uuid,
                NullType::UuidSimple => imr::DbType::Uuid,
                NullType::JsonValue => imr::DbType::Binary,
                #[cfg(feature = "postgres-only")]
                NullType::MacAddress => imr::DbType::MacAddress,
                #[cfg(feature = "postgres-only")]
                NullType::IpNetwork => imr::DbType::IpNetwork,
                #[cfg(feature = "postgres-only")]
                NullType::BitVec => imr::DbType::BitVec,
            },
            annotations: annotations.as_imr(),
            source_defined_at: Some(source_defined_at.clone()),
        });
    }
}

/// Check a [`Field`] for correctness by evaluating its [`FieldType`]'s `Check`
///
/// This function is called and its error reported by the `#[derive(Model)]` macro.
pub const fn check<F: Field>() -> Result<(), ConstString<1024>> {
    <<<F::Type as FieldType>::Check as ConstFn<_, _>>::Body<(
        contains::ExplicitAnnotations<F>,
        contains::EffectiveAnnotations<F>,
    )> as Contains<_>>::ITEM
}

/// A field which is stored in db via a single column
pub trait SingleColumnField: Field {
    /// The annotations which are passed to db
    const EFFECTIVE_ANNOTATION: Annotations;

    /// Borrow an instance of the field's type as a [`Value`]
    fn type_as_value(field: &Self::Type) -> Value<'_>;

    /// Convert an instance of the field's type into a static [`Value`]
    fn type_into_value(field: Self::Type) -> Value<'static>;
}
impl<F> SingleColumnField for F
where
    F: Field,
    F::Type: FieldType<Columns = Array<1>>,
{
    const EFFECTIVE_ANNOTATION: Annotations = {
        let [annos] = Self::EFFECTIVE_ANNOTATIONS;
        annos
    };

    fn type_as_value(field: &Self::Type) -> Value<'_> {
        let [value] = field.as_values();
        value
    }

    fn type_into_value(field: Self::Type) -> Value<'static> {
        let [value] = field.into_values();
        value
    }
}

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

impl<I, T, F, P> std::ops::Deref for FieldProxy<I>
where
    T: FieldType,
    F: Field<Type = T> + ContainerField<T, P>,
    P: Path,
    I: FieldProxyImpl<Field = F, Path = P>,
{
    type Target = F::Target;

    fn deref(&self) -> &'static Self::Target {
        ConstNew::REF
    }
}

impl<T, F, P> ContainerField<T, P> for F
where
    T: FieldType,
    F: Field<Type = T> + PathField<T>,
    P: Path<Current = <F::ParentField as Field>::Model>,
{
    type Target = <<F::ChildField as Field>::Model as Model>::Fields<P::Step<F>>;
}

/// Helper structs implementing [`Contains`] to expose
/// - [`Field::NAME`]
/// - [`Field::EXPLICIT_ANNOTATIONS`]
/// - [`Field::EFFECTIVE_ANNOTATIONS`]
mod contains {
    use std::marker::PhantomData;

    use crate::fields::traits::FieldColumns;
    use crate::fields::utils::column_name::ColumnName;
    use crate::fields::utils::const_fn::Contains;
    use crate::internal::field::Field;
    use crate::internal::hmr::annotations::Annotations;

    pub struct ExplicitAnnotations<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<Annotations> for ExplicitAnnotations<F> {
        const ITEM: Annotations = F::EXPLICIT_ANNOTATIONS;
    }

    pub struct EffectiveAnnotations<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<FieldColumns<F::Type, Annotations>> for EffectiveAnnotations<F> {
        const ITEM: FieldColumns<F::Type, Annotations> = F::EFFECTIVE_ANNOTATIONS;
    }

    pub struct Name<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<ColumnName> for Name<F> {
        const ITEM: ColumnName = F::NAME;
    }
}
