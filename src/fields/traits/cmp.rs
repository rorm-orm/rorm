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

macro_rules! impl_FieldEq {
    ($lhs:ty, $rhs:ty, $into_value:expr) => {
        impl<'rhs> FieldEq<'rhs, $rhs> for $lhs {
            type EqCond<A: FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_equals<A: FieldAccess>(access: A, value: $rhs) -> Self::EqCond<A> {
                Binary {
                    operator: BinaryOperator::Equals,
                    fst_arg: Column(access),
                    #[allow(clippy::redundant_closure_call)] // clean way to pass code to a macro
                    snd_arg: $into_value(value),
                }
            }

            type NeCond<A: FieldAccess> = Binary<Column<A>, Value<'rhs>>;
            fn field_not_equals<A: FieldAccess>(access: A, value: $rhs) -> Self::NeCond<A> {
                Binary {
                    operator: BinaryOperator::NotEquals,
                    fst_arg: Column(access),
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
impl_FieldEq!(String, String, |string| Value::String(Cow::Owned(string)));
impl_FieldEq!(Vec<u8>, Vec<u8>, |vec| Value::Binary(Cow::Owned(vec)));
