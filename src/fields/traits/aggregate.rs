//! Marker traits which can be implemented on a [`FieldType`] to allow the usage of various aggregation functions.
//!
//! ## Using
//! The traits don't prodived any methods. Instead use the corresponding method on [`FieldProxy`].

use rorm_db::row::DecodeOwned;

#[cfg(doc)]
use crate::fields::proxy::FieldProxy;
use crate::fields::traits::{Array, FieldType};

/// Marker for [`FieldProxy::count`]
///
/// This is implemented for every [`SingleColumnFieldType`](crate::internal::field::SingleColumnField)
pub trait FieldCount: FieldType {}
impl<T> FieldCount for T where T: FieldType<Columns = Array<1>> {}

/// Marker for [`FieldProxy::sum`]
pub trait FieldSum: FieldType<Columns = Array<1>> {
    /// The aggregation result's type
    ///
    /// If `Self` is not `Option`, then this should be `Option<Self>`.
    /// If `Self` is a `Option`, then this should be just `Self`.
    type Result: DecodeOwned;
}

/// Marker for [`FieldProxy::avg`]
pub trait FieldAvg: FieldType<Columns = Array<1>> {}

/// Marker for [`FieldProxy::max`]
pub trait FieldMax: FieldType<Columns = Array<1>> {
    /// The aggregation result's type
    ///
    /// If `Self` is not `Option`, then this should be `Option<Self>`.
    /// If `Self` is a `Option`, then this should be just `Self`.
    type Result: DecodeOwned;
}

/// Marker for [`FieldProxy::min`]
pub trait FieldMin: FieldType<Columns = Array<1>> {
    /// The aggregation result's type
    ///
    /// If `Self` is not `Option`, then this should be `Option<Self>`.
    /// If `Self` is a `Option`, then this should be just `Self`.
    type Result: DecodeOwned;
}

/// Implements [`FieldSum`] and [`FieldAvg`] for its argument `T` and `Option<T>`
///
/// # Syntax
/// Pass the type to implement as first argument and pass the [`FieldSum::Result`] type with
/// the `sum_result` key:
/// ```compile_fail
/// impl_FieldSum_FieldAvg!(i16, sum_result: i64);
/// ```
#[allow(non_snake_case)]
#[macro_export]
macro_rules! impl_FieldSum_FieldAvg {
    ($arg:ty, sum_result: $ret:ty) => {
        impl $crate::fields::traits::FieldSum for $arg {
            type Result = Option<$ret>;
        }
        impl $crate::fields::traits::FieldSum for Option<$arg> {
            type Result = Option<$ret>;
        }
        impl $crate::fields::traits::FieldAvg for $arg {}
        impl $crate::fields::traits::FieldAvg for Option<$arg> {}
    };
}

/// Implements [`FieldMin`] and [`FieldMax`] for its argument `T` and `Option<T>`.
///
/// (The `Result` will always be `Option<T>`.)
///
/// # Syntax
/// For a type without generics simple pass it as argument:
/// ```fail_compile
/// impl_FieldMin_FieldMax!(String);
/// ```
///
/// If your type is generic, you have to add a dummy `impl<...>` before the type:
/// ```fail_compile
/// impl_FieldMin_FieldMax!(impl<const MAX_LEN: usize, Impl> MaxStr<MAX_LEN, Impl>);
/// ```
#[allow(non_snake_case)]
#[macro_export]
macro_rules! impl_FieldMin_FieldMax {
    (impl<$($generic:ident $( $const_name:ident : $const_type:ty )?),*> $arg:ty) => {
        $crate::impl_FieldMin_FieldMax!(@internal [<$($generic $( $const_name : $const_type )?),*>] $arg);
    };
    ($arg:ty) => {
        $crate::impl_FieldMin_FieldMax!(@internal [] $arg);
    };
    (@internal [$($generic:tt)*] $arg:ty) => {
        impl $($generic)* $crate::fields::traits::FieldMin for $arg
        where
            $arg: $crate::fields::traits::FieldType,
            Option<$arg>: $crate::db::row::DecodeOwned,
        {
            type Result = Option<$arg>;
        }
        impl $($generic)* $crate::fields::traits::FieldMin for Option<$arg>
        where
            Option<$arg>: $crate::fields::traits::FieldType,
            Option<$arg>: $crate::db::row::DecodeOwned,
        {
            type Result = Option<$arg>;
        }
        impl $($generic)* $crate::fields::traits::FieldMax for $arg
        where
            $arg: $crate::fields::traits::FieldType,
            Option<$arg>: $crate::db::row::DecodeOwned,
        {
            type Result = Option<$arg>;
        }
        impl $($generic)* $crate::fields::traits::FieldMax for Option<$arg>
        where
            Option<$arg>: $crate::fields::traits::FieldType,
            Option<$arg>: $crate::db::row::DecodeOwned,
        {
            type Result = Option<$arg>;
        }
    };
}
