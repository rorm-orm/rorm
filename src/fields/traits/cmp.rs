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
pub trait FieldEq<'rhs, Rhs, Any = ()>: FieldType {
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
pub trait FieldOrd<'rhs, Rhs, Any = ()>: FieldType {
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
pub trait FieldLike<'rhs, Rhs, Any = ()>: FieldType {
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
pub trait FieldRegexp<'rhs, Rhs, Any = ()>: FieldType {
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
pub trait FieldIn<'rhs, Rhs, Any = ()>: FieldType {
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
pub trait FieldILike<'rhs, Rhs, Any = ()>: FieldType {
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
