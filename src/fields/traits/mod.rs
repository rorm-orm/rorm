//! Traits defining types which can be used as fields.

pub use generic_array;
use generic_array::{ArrayLength, GenericArray};
use rorm_db::sql::value::NullType;

pub use self::aggregate::*;
pub use self::cmp::*;
use crate::conditions::Value;
use crate::const_fn;
use crate::fields::utils::column_name::ColumnName;
use crate::fields::utils::const_fn::ConstFn;
use crate::internal::const_concat::ConstString;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::hmr::annotations::Annotations;

pub mod aggregate;
pub mod cmp;
pub mod into_value;
pub mod simple;

/// Base trait for types which are allowed as fields in models
pub trait FieldType: 'static {
    /// Array with length specific to the field type
    type Columns: ArrayLength;

    /// The null types representing `Option<Self>` in the database
    ///
    /// This is used to implement `into_values` and `as_values` for `Option<Self>`,
    /// as well as provide the columns' database types to the migrator.
    const NULL: FieldColumns<Self, NullType>;

    /// Construct an array of [`Value`] representing `self` in the database via ownership
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>>;

    /// Construct an array of [`Value`] representing `self` in the database via borrowing
    fn as_values(&self) -> FieldColumns<Self, Value<'_>>;

    /// [`FieldDecoder`] to use for fields of this type
    type Decoder: FieldDecoder<Result = Self>;

    /// Get the columns' names from the field's name
    type GetNames: ConstFn<(ColumnName,), FieldColumns<Self, ColumnName>>;

    /// Get the columns' annotations from the field's annotations
    type GetAnnotations: ConstFn<(Annotations,), FieldColumns<Self, Annotations>>;

    /// Check a field's annotations to be compatible with this type
    ///
    /// The function gets the annotations explicitly set by the model author
    /// as well as the result from [`FieldType::GetAnnotations`].
    type Check: ConstFn<
        (Annotations, FieldColumns<Self, Annotations>),
        Result<(), ConstString<1024>>,
    >;
}
/// Shorthand for constructing an array with the length for the [`FieldType`]'s columns
pub type FieldColumns<F, T> = GenericArray<T, <F as FieldType>::Columns>;

const_fn! {
    /// Iterates over all annotations and sets the `nullable` flag.
    pub fn set_null<const N: usize>(annotations: [Annotations; N]) -> [Annotations; N] {
        let mut annotations = annotations;
        let mut i = 0;
        while i < annotations.len() {
            annotations[i].nullable = true;
            i += 1;
        }
        annotations
    }
}

/// Provides the "default" implementation of [`FieldType`].
///
/// ## Usages
/// - `impl_FieldType!(RustType, NullType, into_value, as_value);`
///     - `RustType` is the type to implement the traits on.
///     - `NullType` is the database type to associate with (variant of [`NullType`](crate::db::sql::value::NullType)).
///     - `into_value` is used to convert `RustType` into a [`Value<'static>`] (must implement `Fn(RustType) -> Value<'static>`).
///     - `as_value` is used to convert `&onstFn<Annotations, >'a RustType` into a [`Value<'a>`] (must implement `Fn(&'_ RustType) -> Value<'_>`).
///       If `RustType` implements `Copy`, `as_value` can be omitted and will use `into_value` instead.
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[macro_export]
macro_rules! impl_FieldType {
    ($type:ty, $null_type:ident) => {
        impl_FieldType!(
            $type,
            $null_type,
            $crate::fields::utils::check::shared_linter_check<
                $crate::fields::traits::generic_array::typenum::U1,
            >
        );
    };
    ($type:ty, $null_type:ident, $Check:ty) => {
        impl $crate::fields::traits::FieldType for $type {
            type Columns = $crate::fields::traits::generic_array::typenum::U1;

            const NULL: $crate::fields::traits::FieldColumns<
                Self,
                $crate::db::sql::value::NullType,
            > = $crate::fields::traits::generic_array::arr![
                $crate::db::sql::value::NullType::$null_type
            ];

            #[inline(always)]
            fn as_values(
                &self,
            ) -> $crate::fields::traits::FieldColumns<Self, $crate::conditions::Value<'_>> {
                use $crate::fields::traits::into_value::IntoValue;
                $crate::fields::traits::generic_array::arr![self.into_value()]
            }

            fn into_values<'a>(
                self,
            ) -> $crate::fields::traits::FieldColumns<Self, $crate::conditions::Value<'a>> {
                use $crate::fields::traits::into_value::IntoValue;
                $crate::fields::traits::generic_array::arr![self.into_value()]
            }

            type Decoder = $crate::crud::decoder::DirectDecoder<Self>;

            type GetAnnotations = $crate::fields::utils::get_annotations::forward_annotations<
                <Self as $crate::fields::traits::FieldType>::Columns,
            >;

            type Check = $Check;

            type GetNames = $crate::fields::utils::get_names::single_column_name;
        }
    };
}
