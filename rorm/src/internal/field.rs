//! defines the [Field] struct which is stored under `Model::F`.

use rorm_declaration::hmr::annotations::AsImr;
use rorm_declaration::hmr::db_type::DbType;
use rorm_declaration::hmr::Source;
use rorm_declaration::imr;
use std::marker::PhantomData;

use crate::annotation_builder::{AnnotationsDescriptor, ImplicitNotNull};
use crate::internal::as_db_type::AsDbType;
use crate::Model;

/// All information about a model's field stored in the type system.
///
/// This trait is implemented by the `#[derive(Model)]` macro on unique unit struct for each of a model's fields.
///
/// ## Motivation
/// Having this is a trait instead of a generic struct (as it used to be),
/// writing generic code is way cleaner.
/// Also thanks to the [FieldProxy] all a field's information is accessible both via types as well as values.
pub trait Field {
    /// The rust data type stored in this field
    type Type: AsDbType;

    /// The data type as which this field is stored in the db
    ///
    /// It might differ from [AsDbType::DbType], when certain attributes (namely `#[rorm(choices)]`) are set.
    type DbType: DbType;

    /// The model this field is part of
    type Model: Model;

    /// This field's position in the model
    const INDEX: usize;

    /// Name of this field
    const NAME: &'static str;

    /// List of annotations set for this field
    type Annotations: AnnotationsDescriptor + ImplicitNotNull + AsImr<Imr = Vec<imr::Annotation>>;

    /// List of annotations set for this field
    const ANNOTATIONS: Self::Annotations;

    /// Optional definition of the location of field in the source code
    const SOURCE: Option<Source>;

    /// Entry point for compile time checks on a single field
    const _CHECK: () = {
        // Annotations
        let mut annotations: rorm_declaration::lints::Annotations = Self::Annotations::FOOTPRINT;
        annotations.not_null = !Self::Type::IS_NULLABLE && !Self::Annotations::IMPLICIT_NOT_NULL;
        annotations.foreign_key = Self::Type::IS_FOREIGN.is_some();
        if let Err(err) = annotations.check() {
            panic!("{}", err);
        }
    };
}

/// Get a [NewField] as imr
pub fn as_imr<F: Field>() -> imr::Field {
    let mut annotations = F::ANNOTATIONS.as_imr();
    if !F::Type::IS_NULLABLE && !F::Annotations::IMPLICIT_NOT_NULL {
        annotations.push(imr::Annotation::NotNull);
    }
    if let Some((table, column)) = F::Type::IS_FOREIGN {
        annotations.push(imr::Annotation::ForeignKey(imr::ForeignKey {
            table_name: table.to_string(),
            column_name: column.to_string(),
        }))
    }
    imr::Field {
        name: F::NAME.to_string(),
        db_type: F::DbType::IMR,
        annotations,
        source_defined_at: F::SOURCE.map(Into::into),
    }
}

/// This struct acts as a proxy exposing type level information from the [Field] trait on the value level.
///
/// On top of that it can be used to keep track of the "path" this field is accessed through, when dealing with relations.
///
/// ## Type as Value
/// In other words [FieldProxy] allows access to things like [Field::NAME] without access to the concrete field type.
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
pub struct FieldProxy<Field, Path>(Field, PhantomData<Path>);
impl<F: Field, P> FieldProxy<F, P> {
    /// Create a new instance
    pub const fn new(field: F) -> Self {
        Self(field, PhantomData)
    }

    /// Convert this field's db primitive value into the actual rust value
    ///
    /// See [AsDbType::from_primitive] for more details.
    pub fn convert_primitive(&self, primitive: <F::Type as AsDbType>::Primitive) -> F::Type {
        F::Type::from_primitive(primitive)
    }

    /// Get the field's database i.e. column name
    pub const fn name(&self) -> &'static str {
        F::NAME
    }

    /// Get the field's position in the Model
    pub const fn index(&self) -> usize {
        F::INDEX
    }

    /// Get the field's annotations
    pub const fn annotations(&self) -> F::Annotations {
        F::ANNOTATIONS
    }
}
