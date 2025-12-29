//! Various helper for implementing the `FieldType` traits

use crate::conditions::{Binary, BinaryOperator, Column, In, InOperator, Value};
use crate::fields::proxy;
use crate::fields::traits::into_value::IntoValue;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::FieldILike;
use crate::fields::traits::{FieldEq, FieldIn, FieldLike, FieldOrd, FieldType};

/// A simpler alternative to [`FieldEq`]
///
/// `Rhs` should implement [`IntoValue`] in order to be useful.
pub trait SimpleFieldEq<Rhs = Self> {}
impl<'rhs, Rhs, T> FieldEq<'rhs, Rhs, private::Private> for T
where
    Rhs: IntoValue<'rhs>,
    T: SimpleFieldEq<Rhs> + FieldType,
{
    type EqCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_equals<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::EqCond<I> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type NeCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_not_equals<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::NeCond<I> {
        Binary {
            operator: BinaryOperator::NotEquals,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }
}

/// A simpler alternative to [`FieldOrd`]
///
/// `Rhs` should implement [`IntoValue`] in order to be useful.
pub trait SimpleFieldOrd<Rhs = Self> {}
impl<'rhs, Rhs, T> FieldOrd<'rhs, Rhs, private::Private> for T
where
    Rhs: IntoValue<'rhs>,
    T: SimpleFieldOrd<Rhs> + FieldType,
{
    type LtCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_less_than<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::LtCond<I> {
        Binary {
            operator: BinaryOperator::Less,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type LeCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_less_equals<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::LeCond<I> {
        Binary {
            operator: BinaryOperator::LessOrEquals,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type GtCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_greater_than<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::GtCond<I> {
        Binary {
            operator: BinaryOperator::Greater,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type GeCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_greater_equals<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::GeCond<I> {
        Binary {
            operator: BinaryOperator::GreaterOrEquals,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }
}

/// A simpler alternative to [`FieldLike`]
///
/// `Rhs` should implement [`IntoValue`] in order to be useful.
pub trait SimpleFieldLike<Rhs = Self> {}
impl<'rhs, Rhs, T> FieldLike<'rhs, Rhs, private::Private> for T
where
    Rhs: IntoValue<'rhs>,
    T: SimpleFieldLike<Rhs> + FieldType,
{
    type LiCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_like<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::LiCond<I> {
        Binary {
            operator: BinaryOperator::Like,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type NlCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_not_like<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::NlCond<I> {
        Binary {
            operator: BinaryOperator::NotLike,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }
}

/// A simpler alternative to [`FieldIn`]
///
/// `Rhs` should implement [`IntoValue`] in order to be useful.
pub trait SimpleFieldIn<RhsItem = Self> {}
impl<'rhs, Rhs, T> FieldIn<'rhs, Rhs, private::Private> for T
where
    Rhs: IntoIterator,
    Rhs::Item: IntoValue<'rhs>,
    T: SimpleFieldIn<Rhs::Item> + FieldType,
{
    type InCond<I: proxy::FieldProxyImpl> = In<Column<I>, Value<'rhs>>;

    fn field_in<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::InCond<I> {
        In {
            operator: InOperator::In,
            fst_arg: Column(field),
            snd_arg: value.into_iter().map(IntoValue::into_value).collect(),
        }
    }

    type NiCond<I: proxy::FieldProxyImpl> = In<Column<I>, Value<'rhs>>;

    fn field_not_in<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::NiCond<I> {
        In {
            operator: InOperator::NotIn,
            fst_arg: Column(field),
            snd_arg: value.into_iter().map(IntoValue::into_value).collect(),
        }
    }
}

/// A simpler alternative to [`FieldILike`]
///
/// `Rhs` should implement [`IntoValue`] in order to be useful.
#[cfg(feature = "postgres-only")]
pub trait SimpleFieldILike<Rhs = Self> {}
#[cfg(feature = "postgres-only")]
impl<'rhs, Rhs, T> FieldILike<'rhs, Rhs, private::Private> for T
where
    Rhs: IntoValue<'rhs>,
    T: SimpleFieldLike<Rhs> + FieldType,
{
    type IliCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_ilike<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::IliCond<I> {
        Binary {
            operator: BinaryOperator::ILike,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }

    type NilCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_not_ilike<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::NilCond<I> {
        Binary {
            operator: BinaryOperator::NotILike,
            fst_arg: Column(field),
            snd_arg: value.into_value(),
        }
    }
}

mod private {
    pub struct Private;
}
