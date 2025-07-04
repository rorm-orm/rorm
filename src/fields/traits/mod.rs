//! Traits defining types which can be used as fields.

use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;

pub use self::aggregate::*;
pub use self::cmp::*;
use crate::conditions::Value;
use crate::crud::decoder::Decoder;
use crate::fields::proxy;
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::fields::utils::column_name::ColumnName;
use crate::fields::utils::const_fn::ConstFn;
use crate::internal::const_concat::ConstString;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::fake_field::FakeField;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::query_context::QueryContext;
use crate::sealed;

pub mod aggregate;
pub mod cmp;
pub mod simple;

/// Base trait for types which are allowed as fields in models
pub trait FieldType: 'static {
    /// Array with length specific to the field type
    type Columns: Columns;

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

    #[doc(hidden)]
    fn is_option<Private: crate::private::Private>() -> bool {
        false
    }
}
/// Shorthand for constructing an array with the length for the [`FieldType`]'s columns
pub type FieldColumns<F, T> = <<F as FieldType>::Columns as Columns>::Array<T>;

/// The trait for the [`FieldType`]'s `Columns` associated type.
///
/// It is implemented by [`Array`] and is equivalent to a fixed length.
pub trait Columns {
    sealed!(trait);

    /// Array of length `NUM` to store columns' information in
    type Array<T>: IntoIterator<Item = T>;

    /// Calls [`array::map`] on the generic array types produced by [`Self::Array<T>`](Self::Array)
    fn map<T, U>(array: Self::Array<T>, f: impl FnMut(T) -> U) -> Self::Array<U>;

    /// The number of columns
    const NUM: usize;
}

/// Implementor of [`Columns`] used to specify the number of a [`FieldType`]'s columns
pub struct Array<const N: usize>;

impl<const N: usize> Columns for Array<N> {
    sealed!(impl);

    type Array<T> = [T; N];

    fn map<T, U>(array: Self::Array<T>, f: impl FnMut(T) -> U) -> Self::Array<U> {
        array.map(f)
    }

    const NUM: usize = N;
}

impl<T: FieldType> FieldType for Option<T> {
    type Columns = T::Columns;

    const NULL: FieldColumns<Self, NullType> = T::NULL;

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        self.map(T::into_values)
            .unwrap_or(T::Columns::map(T::NULL, Value::Null))
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        self.as_ref()
            .map(T::as_values)
            .unwrap_or(T::Columns::map(T::NULL, Value::Null))
    }

    type Decoder = OptionDecoder<T>;
    type GetNames = T::GetNames;
    // Sadly we can't iterate over the array returned by T::GetAnnotations
    // in a const context in order to set nullable.
    // Therefore, we have to resort to "fixing" it at runtime in the `push_imr` function.
    type GetAnnotations = T::GetAnnotations;
    type Check = T::Check;

    fn is_option<Private: crate::private::Private>() -> bool {
        true
    }
}

/// [`FieldDecoder`] for [`Option<T>`]
pub struct OptionDecoder<T: FieldType>(T::Decoder);
impl<T: FieldType> FieldDecoder for OptionDecoder<T> {
    fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>,
    {
        Self(T::Decoder::new::<(FakeField<T, I::Field>, I::Path)>(
            ctx,
            proxy::new(),
        ))
    }
}
impl<T: FieldType> Decoder for OptionDecoder<T> {
    type Result = Option<T>;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_name(row).map(Some).or_else(|error| match error {
            RowError::UnexpectedNull { .. } => Ok(None),
            _ => Err(error),
        })
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_index(row).map(Some).or_else(|error| match error {
            RowError::UnexpectedNull { .. } => Ok(None),
            _ => Err(error),
        })
    }
}

/// Provides the "default" implementation of [`FieldType`].
///
/// ## Usages
/// - `impl_FieldType!(RustType, NullType, into_value, as_value);`
///     - `RustType` is the type to implement the traits on.
///     - `NullType` is the database type to associate with (variant of [`NullType`](crate::db::sql::value::NullType)).
///     - `into_value` is used to convert `RustType` into a [`Value<'static>`] (must implement `Fn(RustType) -> Value<'static>`).
///     - `as_value` is used to convert `&'a RustType` into a [`Value<'a>`] (must implement `Fn(&'_ RustType) -> Value<'_>`).
///       If `RustType` implements `Copy`, `as_value` can be omitted and will use `into_value` instead.
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[macro_export]
macro_rules! impl_FieldType {
    ($type:ty, $null_type:ident, $into_value:expr) => {
        impl_FieldType!($type, $null_type, $into_value, |&value| $into_value(value));
    };
    ($type:ty, $null_type:ident, $into_value:expr, $as_value:expr) => {
        impl_FieldType!(
            $type,
            $null_type,
            $into_value,
            $as_value,
            $crate::fields::utils::check::shared_linter_check<1>
        );
    };
    ($type:ty, $null_type:ident, $into_value:expr, $as_value:expr, $Check:ty) => {
        impl $crate::fields::traits::FieldType for $type {
            type Columns = $crate::fields::traits::Array<1>;

            const NULL: $crate::fields::traits::FieldColumns<
                Self,
                $crate::db::sql::value::NullType,
            > = [$crate::db::sql::value::NullType::$null_type];

            #[inline(always)]
            fn as_values(
                &self,
            ) -> $crate::fields::traits::FieldColumns<Self, $crate::conditions::Value<'_>> {
                #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                [$as_value(self)]
            }

            fn into_values<'a>(
                self,
            ) -> $crate::fields::traits::FieldColumns<Self, $crate::conditions::Value<'a>> {
                #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                [$into_value(self)]
            }

            type Decoder = $crate::crud::decoder::DirectDecoder<Self>;

            type GetAnnotations = $crate::fields::utils::get_annotations::forward_annotations<1>;

            type Check = $Check;

            type GetNames = $crate::fields::utils::get_names::single_column_name;
        }
    };
}
