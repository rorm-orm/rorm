//! Set of traits similar to [`PartialEq`] and [`PartialOrd`] from `std::cmp`
//! which can be implemented on a [`FieldType`] to allow comparing its value with sql.
//!
//! Also contains more traits corresponding to other sql comparison operators.
//!
//! ## Using
//! Don't call the traits' methods directly. Instead use the corresponding method on [`FieldProxy`].
//! Otherwise the assumptions an implementation is allowed to make, might be violated.
//!
//! ## Implementing
//! - Each method takes an [`FieldProxy`]; an implementation may assume that the access' field's type
//!   matches the type the trait is implemented on. This isn't enforced using trait bounds (yet?) to reduce complexity.

use super::FieldType;
use crate::conditions::{Binary, BinaryOperator, Column, Condition};
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::internal::field::{Field, SingleColumnField};

/// Trait for equality comparisons.
///
/// **Read module notes, before using.**
pub trait FieldEq<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldEq::field_equals`]
    type EqCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `==`
    fn field_equals<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::EqCond<I>;

    /// Condition type returned from [`FieldEq::field_not_equals`]
    type NeCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `!=`
    fn field_not_equals<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NeCond<I>;
}

/// Trait for field types that form an order.
///
/// **Read module notes, before using.**
pub trait FieldOrd<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldOrd::field_less_than`]
    type LtCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `<`
    fn field_less_than<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::LtCond<I>;

    /// Condition type returned from [`FieldOrd::field_less_equals`]
    type LeCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `<=`
    fn field_less_equals<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::LeCond<I>;

    /// Condition type returned from [`FieldOrd::field_greater_than`]
    type GtCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `<`
    fn field_greater_than<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::GtCond<I>;

    /// Condition type returned from [`FieldOrd::field_greater_equals`]
    type GeCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `>=`
    fn field_greater_equals<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs)
        -> Self::GeCond<I>;
}

/// Trait for field types to implement sql's `LIKE` comparison.
///
/// **Read module notes, before using.**
pub trait FieldLike<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldLike::field_like`]
    type LiCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `LIKE`
    fn field_like<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::LiCond<I>;

    /// Condition type returned from [`FieldLike::field_not_like`]
    type NlCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `NOT LIKE`
    fn field_not_like<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NlCond<I>;
}

/// Trait for field types to implement sql's `REGEXP` comparison.
///
/// **Read module notes, before using.**
pub trait FieldRegexp<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldRegexp::field_regexp`]
    type ReCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `REGEXP`
    fn field_regexp<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::ReCond<I>;

    /// Condition type returned from [`FieldRegexp::field_not_regexp`]
    type NrCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `NOT REGEXP`
    fn field_not_regexp<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NrCond<I>;
}

/// Trait for field types to implement sql's `IN` comparison.
///
/// **Read module notes, before using.**
pub trait FieldIn<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldRegexp::field_in`]
    type InCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `IN`
    fn field_in<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::InCond<I>;

    /// Condition type returned from [`FieldRegexp::field_not_in`]
    type NiCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `NOT IN`
    fn field_not_in<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NiCond<I>;
}

/// Trait for field types to implement postgresql's `ILIKE` comparison.
///
/// **Read module notes, before using.**
#[cfg(feature = "postgres-only")]
pub trait FieldILike<'rhs, Rhs: 'rhs, Any = ()>: FieldType {
    /// Condition type returned from [`FieldLike::field_ilike`]
    type IliCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `LIKE`
    fn field_ilike<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::IliCond<I>;

    /// Condition type returned from [`FieldLike::field_not_ilike`]
    type NilCond<I: FieldProxyImpl>: Condition<'rhs>;

    /// Compare the field to another value using `NOT LIKE`
    fn field_not_ilike<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NilCond<I>;
}

// TODO: null check, BETWEEN, IN

/// Provides the "default" implementation of [`FieldEq`].
///
/// It expects a "usual" impl block
/// whose body is a closure which converts the `Rhs` into a [`Value`]
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[deprecated(note = "Use SimpleFieldEq instead")]
#[macro_export]
macro_rules! impl_FieldEq {
    (impl<'rhs $(, $generic:ident $( $const_name:ident : $const_type:ty )?)*> FieldEq<'rhs, $rhs:ty $(, $any:ty)?> for $lhs:ty $(where $( $bound_left:path : $bound_right:path ,)*)? { $into_value:expr }) => {
        impl<'rhs $(, $generic $($const_name : $const_type)?)*> $crate::fields::traits::cmp::FieldEq<'rhs, $rhs $(, $any)?> for $lhs
        where
            $lhs: $crate::fields::traits::FieldType,
            $($( $bound_left : $bound_right ,)*)?
        {
            type EqCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_equals<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs
            ) -> Self::EqCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::Equals,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type NeCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_not_equals<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs
            ) -> Self::NeCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::NotEquals,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }
        }
    };
}

// Impl FieldEq<FieldProxy> iff FieldEq<Self>
impl<'rhs, I2, T> FieldEq<'rhs, FieldProxy<I2>> for T
where
    T: FieldEq<'rhs, T>,
    I2: FieldProxyImpl<Field: Field<Type = T> + SingleColumnField>,
{
    type EqCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;

    fn field_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::EqCond<I> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }

    type NeCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;

    fn field_not_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::NeCond<I> {
        Binary {
            operator: BinaryOperator::NotEquals,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }
}

/// Provides the "default" implementation of [`FieldOrd`].
///
/// It takes
/// - the left hand side type i.e. type to implement on
/// - the right hand side (use `'rhs` a lifetime if required)
/// - a closure to convert the right hand side into a [`Value`]
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[macro_export]
macro_rules! impl_FieldOrd {
    ($lhs:ty, $rhs:ty, $into_value:expr) => {
        impl<'rhs> $crate::fields::traits::cmp::FieldOrd<'rhs, $rhs> for $lhs {
            type LtCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_less_than<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs,
            ) -> Self::LtCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::Less,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type LeCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_less_equals<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs,
            ) -> Self::LeCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::LessOrEquals,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type GtCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_greater_than<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs,
            ) -> Self::GtCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::Greater,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type GeCond<I: $crate::fields::proxy::FieldProxyImpl>
                = $crate::conditions::Binary<
                    $crate::conditions::Column<I>,
                    $crate::conditions::Value<'rhs>,
                >;
            fn field_greater_equals<I: $crate::fields::proxy::FieldProxyImpl>(
                field: $crate::fields::proxy::FieldProxy<I>,
                value: $rhs,
            ) -> Self::GeCond<I> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::GreaterOrEquals,
                    fst_arg: $crate::conditions::Column(field),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }
        }
    };
}

// Impl FieldOrd<FieldProxy> iff FieldOrd<Self>
impl<'rhs, I2, T> FieldOrd<'rhs, FieldProxy<I2>> for T
where
    T: FieldOrd<'rhs, T>,
    I2: FieldProxyImpl<Field: Field<Type = T> + SingleColumnField>,
{
    type LtCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;
    fn field_less_than<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::LtCond<I> {
        Binary {
            operator: BinaryOperator::Less,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }

    type LeCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;
    fn field_less_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::LeCond<I> {
        Binary {
            operator: BinaryOperator::LessOrEquals,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }

    type GtCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;
    fn field_greater_than<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::GtCond<I> {
        Binary {
            operator: BinaryOperator::Greater,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }

    type GeCond<I: FieldProxyImpl> = Binary<Column<I>, Column<I2>>;
    fn field_greater_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: FieldProxy<I2>,
    ) -> Self::GeCond<I> {
        Binary {
            operator: BinaryOperator::GreaterOrEquals,
            fst_arg: Column(field),
            snd_arg: Column(value),
        }
    }
}
