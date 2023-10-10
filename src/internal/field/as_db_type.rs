//! defines and implements the [`AsDbType`] trait.

use fancy_const::const_fn;
use rorm_db::row::DecodeOwned;

use crate::internal::const_concat::ConstString;
use crate::internal::field::FieldType;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;

/// This trait maps rust types to database types
///
/// I.e. it specifies which datatypes are allowed on model's fields.
pub trait AsDbType: FieldType + Sized {
    /// A type which can be retrieved from the db and then converted into Self.
    type Primitive: DecodeOwned;

    /// The database type as defined in the Intermediate Model Representation
    type DbType: DbType;

    /// Annotations implied by this type
    const IMPLICIT: Option<Annotations> = None;
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

            fn get_imr<F: $crate::internal::field::Field<Type = Self>>() -> Self::Columns<$crate::internal::imr::Field> {
                use $crate::internal::hmr::AsImr;
                [$crate::internal::imr::Field {
                    name: F::NAME.to_string(),
                    db_type: <<$type as $crate::internal::field::as_db_type::AsDbType>::DbType as $crate::internal::hmr::db_type::DbType>::IMR,
                    annotations: F::EFFECTIVE_ANNOTATIONS[0].as_imr(),
                    source_defined_at: F::SOURCE.map(|s| s.as_imr()),
                }]
            }

            type Decoder = $decoder;

            type AnnotationsModifier<F: $crate::internal::field::Field<Type = Self>> = $crate::internal::field::modifier::MergeAnnotations<Self>;

            type GetNames = $crate::internal::field::as_db_type::SingleName;

            type GetAnnotations = $crate::internal::field::as_db_type::SingleAnnotationsWithNull;

            type Check = $crate::internal::field::as_db_type::SingleCheck<<$type as $crate::internal::field::as_db_type::AsDbType>::DbType>;
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
        }
    };
    ($type:ty, $db_type:ty, $into_value:expr) => {
        impl_AsDbType!($type, $db_type, $into_value, |&value| $into_value(value));
    };
    ($type:ty, $db_type:ty, $into_value:expr, $as_value:expr) => {
        impl $crate::fields::traits::FieldType for $type {
            type Columns<T> = [T; 1];

            #[inline(always)]
            fn as_values(&self) -> Self::Columns<$crate::conditions::Value<'_>> {
                #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                [$as_value(self)]
            }

            fn into_values(self) -> Self::Columns<$crate::conditions::Value<'static>> {
                #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                [$into_value(self)]
            }

            fn get_imr<F: $crate::internal::field::Field<Type = Self>>() -> Self::Columns<$crate::internal::imr::Field> {
                use $crate::internal::hmr::AsImr;
                [$crate::internal::imr::Field {
                    name: F::NAME.to_string(),
                    db_type: <$db_type as $crate::internal::hmr::db_type::DbType>::IMR,
                    annotations: F::EFFECTIVE_ANNOTATIONS[0].as_imr(),
                    source_defined_at: F::SOURCE.map(|s| s.as_imr()),
                }]
            }

            type Decoder = $crate::crud::decoder::DirectDecoder<Self>;

            type AnnotationsModifier<F: $crate::internal::field::Field<Type = Self>> = $crate::internal::field::modifier::MergeAnnotations<Self>;

            type GetNames = $crate::internal::field::as_db_type::SingleName;

            type GetAnnotations = $crate::internal::field::as_db_type::SingleAnnotations;

            type Check = $crate::internal::field::as_db_type::SingleCheck<$db_type>;
        }

        impl $crate::internal::field::as_db_type::AsDbType for $type {
            type Primitive = Self;

            type DbType = $db_type;
        }

        impl_AsDbType!(Option<$type>, $crate::crud::decoder::DirectDecoder<Self>);
    };
}

const_fn! {
    pub fn SingleName(name: &'static str) -> [&'static str; 1] {
        [name]
    }
}

const_fn! {
    pub fn SingleAnnotations(annos: Annotations) -> [Annotations; 1] {
        [annos]
    }
}

const_fn! {
    pub fn SingleAnnotationsWithNull(annos: Annotations) -> [Annotations; 1] {
        let mut annos = annos;
        annos.nullable = true;
        [annos]
    }
}

const_fn! {
    /// - ensures all annotations required by `D` are set
    /// - runs the shared linter from `rorm-declaration`
    pub fn SingleCheck<D: DbType>(_explicit: Annotations, [effective]: [Annotations; 1]) -> Result<(), ConstString<1024>> {
        'result: {
            // Are required annotations set?
            let mut required = D::REQUIRED;
            while let [head, tail @ ..] = required {
                required = tail;
                if !effective.is_set(head) {
                    break 'result Err(ConstString::error(&[
                        "missing annotation: ",
                        head.as_str(),
                    ]));
                }
            }

            // Run the annotations lint shared with rorm-cli
            let annotations = effective.as_lint();
            if let Err(err) = annotations.check() {
                break 'result Err(ConstString::error(&["invalid annotations: ", err]));
            }

            Ok(())
        }
    }
}
