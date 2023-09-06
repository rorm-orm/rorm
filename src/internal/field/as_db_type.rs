//! defines and implements the [`AsDbType`] trait.

use std::borrow::Cow;

use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::internal::field::{kind, FieldType};
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

/// Provides the "default" implementation of [`AsDbType`] and [`FieldType`] of kind `AsDbType`.
///
/// ## Usages
/// - `impl_as_db_type!(RustType, DbType, into_value, as_value);`
///     - `RustType` is the type to implement the traits on.
///     - `DbType` is the database type to associate with (must implement [`DbType`]).
///     - `into_value` is used to convert `RustType` into a [`Value<'static>`] (must implement `Fn(RustType) -> Value<'static>`).
///     - `as_value` is used to convert `&'a RustType` into a [`Value<'a>`] (must implement `Fn(&'_ RustType) -> Value<'_>`).
///       If `RustType` implements `Copy`, `as_value` can be omitted and will use `into_value` instead.
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[macro_export]
macro_rules! impl_AsDbType {
    (Option<$type:ty>, $decoder:ty) => {
        impl $crate::fields::traits::FieldType for Option<$type> {
            type Kind = $crate::internal::field::kind::AsDbType;
            type Columns<T> = [T; 1];

            fn into_values(self) -> Self::Columns<$crate::conditions::Value<'static>> {
                self.map(<$type>::into_values)
                    .unwrap_or([Value::Null(<<$type as $crate::internal::field::as_db_type::AsDbType>::DbType as $crate::internal::hmr::db_type::DbType>::NULL_TYPE)])
            }

            fn as_values(&self) -> Self::Columns<$crate::conditions::Value<'_>> {
                self.as_ref()
                    .map(<$type>::as_values)
                    .unwrap_or([Value::Null(<<$type as $crate::internal::field::as_db_type::AsDbType>::DbType as $crate::internal::hmr::db_type::DbType>::NULL_TYPE)])
            }

            type Decoder = $decoder;

            type AnnotationsModifier<F: $crate::internal::field::RawField<Type = Self>> = $crate::internal::field::modifier::MergeAnnotations<Self>;
        }

        impl $crate::internal::field::as_db_type::AsDbType for Option<$type> {
            type Primitive = Option<<$type as $crate::internal::field::as_db_type::AsDbType>::Primitive>;
            type DbType = <$type as $crate::internal::field::as_db_type::AsDbType>::DbType;

            const IMPLICIT: Option<$crate::internal::hmr::annotations::Annotations> = {
                let mut annos = if let Some(annos) = <$type as $crate::internal::field::as_db_type::AsDbType>::IMPLICIT {
                    annos
                } else {
                    $crate::internal::hmr::annotations::Annotations::empty()
                };
                annos.nullable = true;
                Some(annos)
            };

            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive.map(<$type as $crate::internal::field::as_db_type::AsDbType>::from_primitive)
            }
        }
    };
    ($type:ty, $db_type:ty, $into_value:expr) => {
        impl_AsDbType!($type, $db_type, $into_value, |&value| $into_value(value));
    };
    ($type:ty, $db_type:ty, $into_value:expr, $as_value:expr) => {
        impl $crate::fields::traits::FieldType for $type {
            type Kind = $crate::internal::field::kind::AsDbType;
            type Columns<T> = [T; 1];

            #[inline(always)]
            fn as_values(&self) -> Self::Columns<$crate::conditions::Value<'_>> {
                [$as_value(self)]
            }

            fn into_values(self) -> Self::Columns<$crate::conditions::Value<'static>> {
                [$into_value(self)]
            }

            type Decoder = $crate::crud::decoder::DirectDecoder<Self>;

            type AnnotationsModifier<F: $crate::internal::field::RawField<Type = Self>> = $crate::internal::field::modifier::MergeAnnotations<Self>;
        }

        impl $crate::internal::field::as_db_type::AsDbType for $type {
            type Primitive = Self;

            type DbType = $db_type;

            #[inline(always)]
            fn from_primitive(primitive: Self::Primitive) -> Self {
                primitive
            }
        }

        impl_AsDbType!(Option<$type>, $crate::crud::decoder::DirectDecoder<Self>);
    };
}
impl_AsDbType!(i16, db_type::Int16, Value::I16);
impl_AsDbType!(i32, db_type::Int32, Value::I32);
impl_AsDbType!(i64, db_type::Int64, Value::I64);
impl_AsDbType!(f32, db_type::Float, Value::F32);
impl_AsDbType!(f64, db_type::Double, Value::F64);
impl_AsDbType!(bool, db_type::Boolean, Value::Bool);
impl_AsDbType!(
    Vec<u8>,
    db_type::Binary,
    |b| Value::Binary(Cow::Owned(b)),
    |b| { Value::Binary(Cow::Borrowed(b)) }
);
impl_AsDbType!(
    String,
    db_type::VarChar,
    |s| Value::String(Cow::Owned(s)),
    |s| { Value::String(Cow::Borrowed(s)) }
);
