//! the various field traits and their proxy.

use std::marker::PhantomData;

use rorm_db::row::RowIndex;
use rorm_db::{Error, Row};
use rorm_declaration::imr;

use crate::conditions::Value;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::{DbType, OptionDbType};
use crate::internal::hmr::{AsImr, Source};
use crate::internal::relation_path::{Path, PathStep};
use crate::model::{ConstNew, Model};
use crate::{const_panic, declare_type_option, sealed, ForeignModel};

pub mod as_db_type;
pub mod foreign_model;
use as_db_type::AsDbType;

/// Little hack to constraint [RawField::RawType] to be the same as [Field::Type] while adding additional constraints.
///
/// **Remember `Self` is always the identical type as `T`!**
pub trait Identical<T>: Into<T> + From<T> {
    sealed!();

    /// "Convert" a reference of `Self` into `T`
    fn as_t_ref(&self) -> &T;

    /// "Convert" a reference of `T` into `Self`
    fn as_self_ref(t: &T) -> &Self;
}
impl<T> Identical<T> for T {
    fn as_t_ref(&self) -> &T {
        self
    }

    fn as_self_ref(t: &T) -> &Self {
        t
    }
}

/// Marker for some field which is part of and interacts with this orm, but isn't actually present in the database
pub struct Pseudo;
/// Marker for some field which corresponds to a column in the database
pub struct Column;

/// Marker trait for the two kinds of fields:
/// - A [Column] field is corresponds to a column in the database.
/// - A [Pseudo] field is something which is part of and interacts with this orm, but isn't actually present in the database.
pub trait FieldKind {
    sealed!();
}
impl FieldKind for Pseudo {}
impl FieldKind for Column {}

/// The type of field allowed on models
pub trait FieldType {
    /// The kind of field this type declares
    type Kind: FieldKind;
}
impl<T: AsDbType> FieldType for T {
    type Kind = Column;
}

/// This trait is implemented by the `#[derive(Model)]` macro on unique unit struct for each of a model's fields.
///
/// It contains all the information a model's author provides on the field.
///
/// This trait itself doesn't do much, but it forms the basis to implement the other traits.
pub trait RawField: 'static {
    /// The field's kind which is determined by its [type](RawField::RawType)
    type Kind: FieldKind;

    /// The type stored in the model's field
    type RawType: FieldType<Kind = Self::Kind>;

    /// An optionally set explicit db type
    type ExplicitDbType: OptionDbType;

    /// An optionally set related field
    type RelatedField: OptionRawField;

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
}

declare_type_option!(OptionRawField, RawField);

/// A [RawField] of kind [Column]
pub trait Field: RawField<Kind = Column> {
    sealed!();

    /// The rust data type stored in this field
    type Type: AsDbType + Identical<Self::RawType>;

    /// The data type as which this field is stored in the db
    ///
    /// It might differ from [AsDbType::DbType], when certain attributes (namely `#[rorm(choices)]`) are set.
    type DbType: DbType;

    /// List of the actual annotations
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

    /// Entry point for compile time checks on a single field
    ///
    /// It is "used" in [FieldProxy::new] to force the compiler to evaluate it.
    const CHECK: usize = {
        // Annotations
        let mut annotations = Self::ANNOTATIONS.as_lint();
        annotations.not_null = !Self::Type::IS_NULLABLE && !Self::ANNOTATIONS.implicit_not_null();
        annotations.foreign_key = Self::Type::IS_FOREIGN.is_some();
        if let Err(err) = annotations.check() {
            const_panic!(&[
                Self::Model::TABLE,
                ".",
                Self::NAME,
                " has invalid annotations: ",
                err
            ]);
        }
        Self::INDEX
    };
}
impl<T: AsDbType, F: RawField<RawType = T, Kind = Column>> Field for F {
    type Type = T;
    type DbType = <F::ExplicitDbType as OptionDbType>::UnwrapOr<<T as AsDbType>::DbType>;
}

/// A common interface unifying the [RawFields](RawField) of various [FieldKinds](FieldKind)
pub trait AbstractField<K: FieldKind = <Self as RawField>::Kind>: RawField {
    sealed!();

    /// Get the field in the intermediate model representation
    ///
    /// Since pseudo field need the same interface this method might return nothing.
    fn imr() -> Option<imr::Field> {
        None
    }

    /// Get an instance of the field's type from a row
    fn get_from_row(row: &Row, index: impl RowIndex) -> Result<Self::RawType, Error>;

    /// Convert a reference to a raw value into a db value
    fn get_value(_value: &Self::RawType) -> Option<Value> {
        None
    }

    /// The column name which stores this field
    const DB_NAME: Option<&'static str> = None;

    /// The list of annotations, if this field is relevant to the database.
    const DB_ANNOTATIONS: Option<Annotations> = None;
}
impl<F: Field> AbstractField<Column> for F {
    fn imr() -> Option<imr::Field> {
        let mut annotations = F::ANNOTATIONS.as_imr();
        if !F::Type::IS_NULLABLE && !F::ANNOTATIONS.implicit_not_null() {
            annotations.push(imr::Annotation::NotNull);
        }
        if let Some((table, column)) = F::Type::IS_FOREIGN {
            annotations.push(imr::Annotation::ForeignKey(imr::ForeignKey {
                table_name: table.to_string(),
                column_name: column.to_string(),
                on_delete: F::ANNOTATIONS.on_delete.unwrap_or_default(),
                on_update: F::ANNOTATIONS.on_update.unwrap_or_default(),
            }))
        }
        Some(imr::Field {
            name: F::NAME.to_string(),
            db_type: F::DbType::IMR,
            annotations,
            source_defined_at: F::SOURCE.map(|s| s.as_imr()),
        })
    }

    fn get_from_row(row: &Row, index: impl RowIndex) -> Result<Self::RawType, Error> {
        Ok(<Self as Field>::Type::from_primitive(row.get(index)?).into())
    }

    fn get_value(value: &Self::RawType) -> Option<Value> {
        Some(<<Self as Field>::Type as Identical<Self::RawType>>::as_self_ref(value).as_primitive())
    }

    const DB_NAME: Option<&'static str> = Some(F::NAME);

    const DB_ANNOTATIONS: Option<Annotations> = {
        // "Use" the CHECK constant to force the compiler to evaluate it.
        let _check: usize = F::CHECK;
        Some(F::ANNOTATIONS)
    };
}
impl<T: Default, F: RawField<Kind = Pseudo, RawType = T>> AbstractField<Pseudo> for F {
    fn get_from_row(_row: &Row, _index: impl RowIndex) -> Result<Self::RawType, Error> {
        Ok(T::default())
    }
}

/// This struct acts as a proxy exposing type level information from the [Field] trait on the value level.
///
/// On top of that it can be used to keep track of the "path" this field is accessed through, when dealing with relations.
///
/// ## Type as Value
/// In other words [FieldProxy] allows access to things like [RawField::NAME] without access to the concrete field type.
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
pub struct FieldProxy<Field, Path>(PhantomData<(Field, Path)>);
impl<F: RawField, P> FieldProxy<F, P> {
    /// Create a new instance
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    /// Get the field's position in the Model
    pub const fn index(&self) -> usize {
        F::INDEX
    }
}
impl<F: Field, P> FieldProxy<F, P> {
    /// Get the field's annotations
    pub const fn annotations(&self) -> Annotations {
        F::ANNOTATIONS
    }
}
impl<F: AbstractField, P> FieldProxy<F, P> {
    /// Get the field's database i.e. column name
    pub const fn name(&self) -> Option<&'static str> {
        F::DB_NAME
    }

    /// Get an instance of the field's type from a row
    pub fn get_from_row(
        &self,
        row: &Row,
        index: Option<impl RowIndex>,
    ) -> Result<F::RawType, Error> {
        if let Some(index) = index {
            F::get_from_row(row, index)
        } else {
            F::get_from_row(row, F::NAME)
        }
    }

    /// Get a condition value from a reference
    pub fn get_value<'a>(&self, value: &'a F::RawType) -> Option<Value<'a>> {
        F::get_value(value)
    }
}

impl<M: Model, F: Field<Type = ForeignModel<M>>, P: Path> FieldProxy<F, P> {
    /// Get the foreign model's fields keeping track where you came from
    pub const fn fields(&self) -> M::Fields<PathStep<F, P>> {
        M::Fields::NEW
    }
}
