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

use rorm_declaration::imr;

use crate::conditions::Value;
use crate::fields::proxy::FieldProxy;
use crate::fields::proxy::FieldProxyImpl;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::{self, AsImr, Source};
use crate::internal::relation_path::{Path, PathField, PathFieldType};
use crate::model::Model;

pub mod cascading_foreign_model;
pub mod foreign_model;
pub mod multi_column;

use crate::fields::traits::{Array, Columns, FieldColumns, FieldType};
use crate::fields::utils::column_name::ColumnName;
use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::const_concat::ConstString;
use crate::internal::ConstRef;

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

    for ((name, annotations), null_type) in names.into_iter().zip(annotations).zip(db_types) {
        imr.push(imr::Field {
            name: name.as_str().to_string(),
            db_type: hmr::db_type(null_type),
            annotations: annotations.as_imr(),
            source_defined_at: Some(source_defined_at.clone()),
        });
    }
}

/// Runs the shared linter on a [`Field`]'s columns
///
/// This will check each column's annotations to be valid with each other
/// and valid of the column's type.
///
/// This function is called and its error reported by the `#[derive(Model)]` macro.
#[allow(clippy::result_large_err, reason = "There is no heap in const")]
pub const fn check<F: Field>() -> Result<(), ConstString<1024>> {
    if <<F::Type as FieldType>::Columns as Columns>::NUM > 0 {
        <<<<F::Type as FieldType>::Columns as Columns>::Lint as ConstFn<_, _>>::Body<(
            contains::EffectiveNames<F>,
            contains::NullTypes<F>,
            contains::EffectiveAnnotations<F>,
        )> as Contains<_>>::ITEM
    } else {
        match F::EXPLICIT_ANNOTATIONS {
            Annotations {
                auto_create_time: None,
                auto_update_time: None,
                auto_increment: None,
                choices: None,
                default: None,
                index: None,
                max_length: None,
                on_delete: None,
                on_update: None,
                primary_key: None,
                unique: None,
                nullable: false,
                foreign: None,
            } => Ok(()),
            _ => Err(ConstString::error(&[
                "Field `",
                F::NAME.as_str(),
                "` has no columns and doesn't support any annotation",
            ])),
        }
    }
}

/// A field which is stored in db via a single column
///
/// This trait is a trait alias with a few additional utility methods.
pub trait SingleColumnField: Field<Type: FieldType<Columns = Array<1>>> {
    /// The annotations which are passed to db
    const EFFECTIVE_ANNOTATION: Annotations = {
        let [annos] = Self::EFFECTIVE_ANNOTATIONS;
        annos
    };

    /// Borrow an instance of the field's type as a [`Value`]
    fn type_as_value(field: &Self::Type) -> Value<'_> {
        let [value] = field.as_values();
        value
    }

    /// Convert an instance of the field's type into a static [`Value`]
    fn type_into_value(field: Self::Type) -> Value<'static> {
        let [value] = field.into_values();
        value
    }
}
impl<F> SingleColumnField for F where F: Field<Type: FieldType<Columns = Array<1>>> {}

/// A [`FieldType`] whose fields' proxies should implement [`Deref`](std::ops::Deref) to some collection of fields.
///
/// This collection's meaning depends on the field type
/// - For [`BackRef`](crate::fields::types::BackRef) and [`ForeignModel`](crate::fields::types::ForeignModelByField),
///   it is their related model's fields
/// - For multi-column fields, it is their "contained" fields
pub trait ContainerFieldType<F, P>
where
    Self: FieldType,
    F: Field<Type = Self>,
    P: Path<Current = F::Model>,
{
    /// Struct of contained fields
    type Target: ConstRef;
}

impl<I, F, P> std::ops::Deref for FieldProxy<I>
where
    I: FieldProxyImpl<Field = F, Path = P>,
    F: Field<Type: ContainerFieldType<F, P>>,
    P: Path<Current = F::Model>,
{
    type Target = <F::Type as ContainerFieldType<F, P>>::Target;

    fn deref(&self) -> &'static Self::Target {
        ConstRef::REF
    }
}

impl<T, F, P> ContainerFieldType<F, P> for T
where
    // trait requirements
    Self: FieldType,
    F: Field<Type = Self>,
    P: Path<Current = F::Model>,
    // further restrictions
    F: PathField<ParentField: Field<Model = F::Model>>, // the parent of the join is on the same model as the current field
    Self: PathFieldType, // this redundant restriction is required to distinguish this "for T" impl from a user defined type (which cannot impl PathFieldType)
{
    type Target = <<F::ChildField as Field>::Model as Model>::Fields<P::Step<F>>;
}

/// Helper structs implementing [`Contains`] to expose
/// - [`Field::NAME`]
/// - [`Field::EXPLICIT_ANNOTATIONS`]
/// - [`Field::EFFECTIVE_ANNOTATIONS`]
/// - [`FieldType::NULL`]
mod contains {
    use std::marker::PhantomData;

    use crate::fields::traits::{FieldColumns, FieldType};
    use crate::fields::utils::column_name::ColumnName;
    use crate::fields::utils::const_fn::Contains;
    use crate::internal::field::Field;
    use crate::internal::hmr::annotations::Annotations;
    use rorm_db::sql::value::NullType;

    pub struct ExplicitAnnotations<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<Annotations> for ExplicitAnnotations<F> {
        const ITEM: Annotations = F::EXPLICIT_ANNOTATIONS;
    }

    pub struct EffectiveNames<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<FieldColumns<F::Type, ColumnName>> for EffectiveNames<F> {
        const ITEM: FieldColumns<F::Type, ColumnName> = F::EFFECTIVE_NAMES;
    }

    pub struct NullTypes<F: Field>(PhantomData<F>);
    impl<F: Field> Contains<FieldColumns<F::Type, NullType>> for NullTypes<F> {
        const ITEM: FieldColumns<F::Type, NullType> = <F::Type as FieldType>::NULL;
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
