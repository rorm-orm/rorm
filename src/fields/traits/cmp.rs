//! Set of traits similar to [`PartialEq`] and [`PartialOrd`] from `std::cmp`
//! which can be implemented on a [`FieldType`] to allow comparing its value with sql.
//!
//! Also contains more traits corresponding to other sql comparison operators.
//!
//! ## Using
//! Don't call the traits' methods directly. Instead use the corresponding method on [`FieldAccess`].
//! Otherwise the assumptions an implementation is allowed to make, might be violated.
//!
//! ## Implementing
//! - Each method takes an [`FieldAccess`]; an implementation may assume that the access' field's type
//!   matches the type the trait is implemented on. This isn't enforced using trait bounds (yet?) to reduce complexity.

use std::borrow::Cow;

use super::FieldType;
use crate::conditions::{Binary, BinaryOperator, Column, Condition, Value};
use crate::internal::field::access::FieldAccess;
use crate::internal::field::{FieldProxy, RawField, SingleColumnField};
use crate::internal::relation_path::Path;

/// Trait for equality comparisons.
///
/// **Read module notes, before using.**
pub trait FieldEq<'rhs, Rhs: 'rhs>: FieldType {
    /// Condition type returned from [`FieldEq::field_equals`]
    type EqCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `==`
    fn field_equals<A: FieldAccess>(access: A, value: Rhs) -> Self::EqCond<A>;

    /// Condition type returned from [`FieldEq::field_not_equals`]
    type NeCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `!=`
    fn field_not_equals<A: FieldAccess>(access: A, value: Rhs) -> Self::NeCond<A>;
}

/// Trait for field types that form an order.
///
/// **Read module notes, before using.**
pub trait FieldOrd<'rhs, Rhs: 'rhs>: FieldEq<'rhs, Rhs> {
    /// Condition type returned from [`FieldOrd::field_less_than`]
    type LtCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `<`
    fn field_less_than<A: FieldAccess>(access: A, value: Rhs) -> Self::LtCond<A>;

    /// Condition type returned from [`FieldOrd::field_less_equals`]
    type LeCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `<=`
    fn field_less_equals<A: FieldAccess>(access: A, value: Rhs) -> Self::LeCond<A>;

    /// Condition type returned from [`FieldOrd::field_greater_than`]
    type GtCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `<`
    fn field_greater_than<A: FieldAccess>(access: A, value: Rhs) -> Self::GtCond<A>;

    /// Condition type returned from [`FieldOrd::field_greater_equals`]
    type GeCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `>=`
    fn field_greater_equals<A: FieldAccess>(access: A, value: Rhs) -> Self::GeCond<A>;
}

/// Trait for field types to implement sql's `LIKE` comparison.
///
/// **Read module notes, before using.**
pub trait FieldLike<'rhs, Rhs: 'rhs> {
    /// Condition type returned from [`FieldLike::field_like`]
    type LiCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `LIKE`
    fn field_like<A: FieldAccess>(access: A, value: Rhs) -> Self::LiCond<A>;

    /// Condition type returned from [`FieldLike::field_not_like`]
    type NlCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `NOT LIKE`
    fn field_not_like<A: FieldAccess>(access: A, value: Rhs) -> Self::NlCond<A>;
}

/// Trait for field types to implement sql's `REGEXP` comparison.
///
/// **Read module notes, before using.**
pub trait FieldRegexp<'rhs, Rhs: 'rhs> {
    /// Condition type returned from [`FieldRegexp::field_regexp`]
    type ReCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `REGEXP`
    fn field_regexp<A: FieldAccess>(access: A, value: Rhs) -> Self::ReCond<A>;

    /// Condition type returned from [`FieldRegexp::field_not_regexp`]
    type NrCond<A: FieldAccess>: Condition<'rhs>;

    /// Compare the field to another value using `NOT REGEXP`
    fn field_not_regexp<A: FieldAccess>(access: A, value: Rhs) -> Self::NrCond<A>;
}

// TODO: null check, BETWEEN, IN

/// Provides the "default" implementation of [`FieldEq`].
///
/// It takes
/// - the left hand side type i.e. type to implement on
/// - the right hand side (use `'rhs` a lifetime if required)
/// - a closure to convert the right hand side into a [`Value`]
#[doc(hidden)]
#[allow(non_snake_case)] // makes it clearer that a trait and which trait is meant
#[macro_export]
macro_rules! impl_FieldEq {
    ($lhs:ty, $rhs:ty, $into_value:expr) => {
        impl<'rhs> $crate::fields::traits::cmp::FieldEq<'rhs, $rhs> for $lhs {
            type EqCond<A: $crate::FieldAccess> = $crate::conditions::Binary<$crate::conditions::Column<A>, $crate::conditions::Value<'rhs>>;
            fn field_equals<A: $crate::FieldAccess>(access: A, value: $rhs) -> Self::EqCond<A> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::Equals,
                    fst_arg: $crate::conditions::Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type NeCond<A: $crate::FieldAccess> = $crate::conditions::Binary<$crate::conditions::Column<A>, $crate::conditions::Value<'rhs>>;
            fn field_not_equals<A: $crate::FieldAccess>(access: A, value: $rhs) -> Self::NeCond<A> {
                $crate::conditions::Binary {
                    operator: $crate::conditions::BinaryOperator::NotEquals,
                    fst_arg: $crate::conditions::Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }
        }
    };
}

impl_FieldEq!(bool, bool, Value::Bool);
impl_FieldEq!(i16, i16, Value::I16);
impl_FieldEq!(i32, i32, Value::I32);
impl_FieldEq!(i64, i64, Value::I64);
impl_FieldEq!(f32, f32, Value::F32);
impl_FieldEq!(f64, f64, Value::F64);

impl_FieldEq!(String, &'rhs str, |s| Value::String(Cow::Borrowed(s)));
impl_FieldEq!(String, &'rhs String, |s| Value::String(Cow::Borrowed(s)));
impl_FieldEq!(String, String, |s| Value::String(Cow::Owned(s)));
impl_FieldEq!(String, Cow<'rhs, str>, Value::String);

impl_FieldEq!(Vec<u8>, &'rhs [u8], |b| Value::Binary(Cow::Borrowed(b)));
impl_FieldEq!(String, &'rhs Vec<u8>, |s| Value::Binary(Cow::Borrowed(s)));
impl_FieldEq!(Vec<u8>, Vec<u8>, |b| Value::Binary(Cow::Owned(b)));
impl_FieldEq!(Vec<u8>, Cow<'rhs, [u8]>, Value::Binary);

// Impl FieldEq<FieldProxy> iff FieldEq<Self>
impl<'rhs, F, P, T> FieldEq<'rhs, FieldProxy<F, P>> for T
where
    T: FieldEq<'rhs, T>,
    F: RawField<Type = T> + SingleColumnField,
    P: Path,
{
    type EqCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;

    fn field_equals<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::EqCond<A> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(access),
            snd_arg: Column(value),
        }
    }

    type NeCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;

    fn field_not_equals<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::NeCond<A> {
        Binary {
            operator: BinaryOperator::NotEquals,
            fst_arg: Column(access),
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
            type LtCond<A: $crate::FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_less_than<A: $crate::FieldAccess>(access: A, value: $rhs) -> Self::LtCond<A> {
                Binary {
                    operator: BinaryOperator::Less,
                    fst_arg: Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type LeCond<A: $crate::FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_less_equals<A: $crate::FieldAccess>(access: A, value: $rhs) -> Self::LeCond<A> {
                Binary {
                    operator: BinaryOperator::LessOrEquals,
                    fst_arg: Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type GtCond<A: FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_greater_than<A: FieldAccess>(access: A, value: $rhs) -> Self::GtCond<A> {
                Binary {
                    operator: BinaryOperator::Greater,
                    fst_arg: Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type GeCond<A: FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_greater_equals<A: FieldAccess>(access: A, value: $rhs) -> Self::GeCond<A> {
                Binary {
                    operator: BinaryOperator::GreaterOrEquals,
                    fst_arg: Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }
        }
    };
}

impl_FieldOrd!(i16, i16, Value::I16);
impl_FieldOrd!(i32, i32, Value::I32);
impl_FieldOrd!(i64, i64, Value::I64);
impl_FieldOrd!(f32, f32, Value::F32);
impl_FieldOrd!(f64, f64, Value::F64);

impl_FieldOrd!(String, &'rhs str, |s| Value::String(Cow::Borrowed(s)));
impl_FieldOrd!(String, String, |s| Value::String(Cow::Owned(s)));
impl_FieldOrd!(String, Cow<'rhs, str>, Value::String);

impl_FieldOrd!(Vec<u8>, &'rhs [u8], |b| Value::Binary(Cow::Borrowed(b)));
impl_FieldOrd!(Vec<u8>, Vec<u8>, |b| Value::Binary(Cow::Owned(b)));
impl_FieldOrd!(Vec<u8>, Cow<'rhs, [u8]>, Value::Binary);

// Impl FieldOrd<FieldProxy> iff FieldOrd<Self>
impl<'rhs, F, P, T> FieldOrd<'rhs, FieldProxy<F, P>> for T
where
    T: FieldOrd<'rhs, T>,
    F: RawField<Type = T> + SingleColumnField,
    P: Path,
{
    type LtCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;
    fn field_less_than<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::LtCond<A> {
        Binary {
            operator: BinaryOperator::Less,
            fst_arg: Column(access),
            snd_arg: Column(value),
        }
    }

    type LeCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;
    fn field_less_equals<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::LeCond<A> {
        Binary {
            operator: BinaryOperator::LessOrEquals,
            fst_arg: Column(access),
            snd_arg: Column(value),
        }
    }

    type GtCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;
    fn field_greater_than<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::GtCond<A> {
        Binary {
            operator: BinaryOperator::Greater,
            fst_arg: Column(access),
            snd_arg: Column(value),
        }
    }

    type GeCond<A: FieldAccess> = Binary<Column<A>, Column<FieldProxy<F, P>>>;
    fn field_greater_equals<A: FieldAccess>(access: A, value: FieldProxy<F, P>) -> Self::GeCond<A> {
        Binary {
            operator: BinaryOperator::GreaterOrEquals,
            fst_arg: Column(access),
            snd_arg: Column(value),
        }
    }
}
