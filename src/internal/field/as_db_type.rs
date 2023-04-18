//! defines and implements the [`AsDbType`] trait.

use std::borrow::Cow;

use chrono::{TimeZone, Utc};
use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::internal::field::{extract_value, kind, FieldType};
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type;
use crate::internal::hmr::db_type::DbType;

/// This trait maps rust types to database types
///
/// I.e. it specifies which datatypes are allowed on model's fields.
pub trait AsDbType: FieldType<Kind = kind::AsDbType> + Sized {
    /// A type which can be retrieved from the db and then converted into Self.
    type Primitive: DecodeOwned;

    /// The database type as defined in the Intermediate Model Representation
    type DbType: DbType;

    /// Annotations implied by this type
    const IMPLICIT: Option<Annotations> = None;

    /// Convert the associated primitive type into `Self`.
    ///
    /// This function allows "non-primitive" types like any [`DbEnum`](rorm_macro::DbEnum) to implement
    /// their decoding without access to the underlying db details (namely `sqlx::Decode`)
    fn from_primitive(primitive: Self::Primitive) -> Self;
}

macro_rules! impl_as_db_type {
    ($type:ty, $db_type:ident, $value_variant:ident $(using $method:ident)?) => {
        impl FieldType for $type {
            type Kind = kind::AsDbType;
            type Columns<'a> = [Value<'a>; 1];

            impl_as_db_type!(impl_as_primitive, $type, $db_type, $value_variant $(using $method)?);
        }
        impl AsDbType for $type {
            type Primitive = Self;

            type DbType = db_type::$db_type;

            #[inline(always)]
            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive
            }
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident) => {
        #[inline(always)]
        fn as_values(&self) -> Self::Columns<'_> {
            [Value::$value_variant(*self)]
        }

        #[inline(always)]
        fn into_values(self) -> Self::Columns<'static> {
            [Value::$value_variant(self)]
        }
    };
    (impl_as_primitive, $type:ty, $db_type:ident, $value_variant:ident using $method:ident) => {
        #[inline(always)]
        fn as_values(&self) -> Self::Columns<'_> {
            [Value::$value_variant(Cow::Borrowed(self.$method()))]
        }

        #[inline(always)]
        fn into_values(self) -> Self::Columns<'static> {
            [Value::$value_variant(Cow::Owned(self))]
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

impl FieldType for chrono::DateTime<Utc> {
    type Kind = kind::AsDbType;

    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [Value::NaiveDateTime(self.naive_utc())]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [Value::NaiveDateTime(self.naive_utc())]
    }
}
impl AsDbType for chrono::DateTime<Utc> {
    type Primitive = chrono::NaiveDateTime;
    type DbType = db_type::DateTime;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Utc.from_utc_datetime(&primitive)
    }
}

impl<T: AsDbType> FieldType for Option<T> {
    type Kind = kind::AsDbType;

    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [if let Some(value) = self {
            let Some(value) = extract_value(value.into_values()) else {
                unreachable!("An AsDbType may only be stored in a single column")
            };
            value
        } else {
            Value::Null(T::DbType::NULL_TYPE)
        }]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [if let Some(value) = self {
            let Some(value) = extract_value(value.as_values()) else {
                unreachable!("An AsDbType may only be stored in a single column")
            };
            value
        } else {
            Value::Null(T::DbType::NULL_TYPE)
        }]
    }
}
impl<T: AsDbType> AsDbType for Option<T> {
    type Primitive = Option<T::Primitive>;
    type DbType = T::DbType;

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
}
