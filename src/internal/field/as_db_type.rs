//! defines and implements the [AsDbType] trait.

use rorm_db::row::DecodeOwned;
use rorm_db::value::NullType;

use crate::conditions::Value;
use crate::internal::field::{kind, FieldType};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;

/// This trait maps rust types to database types
///
/// I.e. it specifies which datatypes are allowed on model's fields.
///
/// The mysterious generic `F` which appears in some places, is a workaround.
/// [ForeignModel] requires the [Field] it is the type of, in order to access its [RelatedField].
/// Instead of creating a whole new process to define a [Field] which has a [RawType] of [ForeignModel],
/// the places which require the [RelatedField] forward the [Field] via this generic `F`.
///
/// [RawType]: crate::internal::field::RawField::Type
/// [ForeignModel]: crate::internal::field::foreign_model::ForeignModel
/// [RelatedField]: crate::internal::field::RawField::RelatedField
pub trait AsDbType: FieldType<Kind = kind::AsDbType> {
    /// A type which can be retrieved from the db and then converted into Self.
    type Primitive: DecodeOwned;

    /// The database type as defined in the Intermediate Model Representation
    type DbType: hmr::db_type::DbType;

    /// Type to pass to rorm-sql for null
    const NULL_TYPE: NullType;

    /// Annotations implied by this type
    const IMPLICIT: Option<Annotations> = None;

    /// Convert the associated primitive type into `Self`.
    ///
    /// This function allows "non-primitive" types like any [DbEnum](rorm_macro::DbEnum) to implement
    /// their decoding without access to the underlying db details (namely `sqlx::Decode`)
    fn from_primitive(primitive: Self::Primitive) -> Self;

    /// Convert a reference to `Self` into the primitive [`Value`] used by our db implementation.
    fn as_primitive(&self) -> Value;
}

macro_rules! impl_as_db_type {
    ($type:ty, $db_type:ident, $value_variant:ident $(using $method:ident)?) => {
        impl FieldType for $type {
            type Kind = kind::AsDbType;
        }
        impl AsDbType for $type {
            type Primitive = Self;

            type DbType = hmr::db_type::$db_type;

            const NULL_TYPE: NullType = NullType::$value_variant;

            #[inline(always)]
            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive
            }

            impl_as_db_type!(impl_as_primitive, $type, $db_type, $value_variant $(using $method)?);
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident) => {
        #[inline(always)]
        fn as_primitive(&self) -> Value {
            Value::$value_variant(*self)
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident using $method:ident) => {
        #[inline(always)]
        fn as_primitive(&self) -> Value {
            Value::$value_variant(self.$method())
        }
    };
}
impl_as_db_type!(chrono::NaiveTime, Time, NaiveTime);
impl_as_db_type!(chrono::NaiveDateTime, DateTime, NaiveDateTime);
impl_as_db_type!(chrono::NaiveDate, Date, NaiveDate);
impl_as_db_type!(i16, Int16, I16);
impl_as_db_type!(i32, Int32, I32);
impl_as_db_type!(i64, Int64, I64);
impl_as_db_type!(f32, Float, F32);
impl_as_db_type!(f64, Double, F64);
impl_as_db_type!(bool, Boolean, Bool);
impl_as_db_type!(Vec<u8>, VarBinary, Binary using as_slice);
impl_as_db_type!(String, VarChar, String using as_str);

impl<T: AsDbType> FieldType for Option<T> {
    type Kind = kind::AsDbType;
}
impl<T: AsDbType> AsDbType for Option<T> {
    type Primitive = Option<T::Primitive>;
    type DbType = T::DbType;

    const NULL_TYPE: NullType = T::NULL_TYPE;

    const IMPLICIT: Option<Annotations> = {
        let mut annos = if let Some(annos) = T::IMPLICIT {
            annos
        } else {
            Annotations::empty()
        };
        annos.nullable = true;
        Some(annos)
    };

    fn from_primitive(primitive: Self::Primitive) -> Self {
        primitive.map(T::from_primitive)
    }

    fn as_primitive(&self) -> Value {
        match self {
            Some(value) => value.as_primitive(),
            None => Value::Null(T::NULL_TYPE),
        }
    }
}
