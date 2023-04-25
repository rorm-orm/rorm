//! defines and implements the [`AsDbType`] trait.

use std::borrow::Cow;

use chrono::{TimeZone, Utc};
use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::internal::field::{kind, FieldType};
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type;
use crate::internal::hmr::db_type::DbType;
use crate::new_converting_decoder;

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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_option_as_db_type {
    ($type:ty, $decoder:ty) => {
        impl FieldType for Option<$type> {
            type Kind = kind::AsDbType;
            type Columns<'a> = [Value<'a>; 1];

            fn into_values(self) -> Self::Columns<'static> {
                self.map(<$type>::into_values)
                    .unwrap_or([Value::Null(<<$type as AsDbType>::DbType as $crate::internal::hmr::db_type::DbType>::NULL_TYPE)])
            }

            fn as_values(&self) -> Self::Columns<'_> {
                self.as_ref()
                    .map(<$type>::as_values)
                    .unwrap_or([Value::Null(<<$type as AsDbType>::DbType as $crate::internal::hmr::db_type::DbType>::NULL_TYPE)])
            }

            type Decoder = $decoder;
        }

        impl AsDbType for Option<$type> {
            type Primitive = Option<<$type as AsDbType>::Primitive>;
            type DbType = <$type as AsDbType>::DbType;

            const IMPLICIT: Option<Annotations> = {
                let mut annos = if let Some(annos) = <$type as AsDbType>::IMPLICIT {
                    annos
                } else {
                    Annotations::empty()
                };
                annos.nullable = true;
                Some(annos)
            };

            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive.map(<$type as AsDbType>::from_primitive)
            }
        }
    };
}
macro_rules! impl_as_db_type {
    ($type:ty, $db_type:ident, $value_variant:ident $(using $method:ident)?) => {
        impl FieldType for $type {
            type Kind = kind::AsDbType;
            type Columns<'a> = [Value<'a>; 1];

            impl_as_db_type!(impl_as_primitive, $type, $db_type, $value_variant $(using $method)?);

            type Decoder = DirectDecoder<Self>;
        }

        impl AsDbType for $type {
            type Primitive = Self;

            type DbType = db_type::$db_type;

            #[inline(always)]
            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive
            }
        }

        impl_option_as_db_type!($type, DirectDecoder<Self>);
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

new_converting_decoder!(
    /// [`FieldDecoder`] for [`chrono::DateTime<Utc>`]
    UtcDateTimeDecoder,
    |value: chrono::NaiveDateTime| -> chrono::DateTime<Utc> { Ok(Utc.from_utc_datetime(&value)) }
);
impl FieldType for chrono::DateTime<Utc> {
    type Kind = kind::AsDbType;

    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [Value::NaiveDateTime(self.naive_utc())]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [Value::NaiveDateTime(self.naive_utc())]
    }

    type Decoder = UtcDateTimeDecoder;
}
impl AsDbType for chrono::DateTime<Utc> {
    type Primitive = chrono::NaiveDateTime;
    type DbType = db_type::DateTime;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Utc.from_utc_datetime(&primitive)
    }
}
new_converting_decoder!(
    /// [`FieldDecoder`] for [`Option<chrono::DateTime<Utc>>`](chrono::DateTime)
    OptionUtcDateTimeDecoder,
    |value: Option<chrono::NaiveDateTime>| -> Option<chrono::DateTime<Utc>> {
        Ok(value.map(|value| Utc.from_utc_datetime(&value)))
    }
);
impl_option_as_db_type!(chrono::DateTime<Utc>, OptionUtcDateTimeDecoder);
