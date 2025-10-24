//! Various helper for implementing the `FieldType` traits

use crate::conditions::{Binary, BinaryOperator, Column, In, InOperator, Value};
use crate::fields::proxy;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::FieldILike;
use crate::fields::traits::{FieldEq, FieldIn, FieldLike, FieldType};

/// A simpler alternative to [`FieldEq`]
///
/// It will convert the `rhs` into a sql value and compare it using the normal
/// equal and not equal operators.
pub trait SimpleFieldEq<'rhs, Rhs, Any = ()> {
    /// Converts the rhs into a sql value
    fn into_value(rhs: Rhs) -> Value<'rhs>;
}
impl<'rhs, Rhs, Any, T> FieldEq<'rhs, Rhs, private::SimpleFieldEq<Any>> for T
where
    Rhs: 'rhs,
    T: SimpleFieldEq<'rhs, Rhs, Any> + FieldType,
{
    type EqCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;
    fn field_equals<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::EqCond<I> {
        Binary {
            operator: BinaryOperator::Equals,
            fst_arg: Column(field),
            snd_arg: Self::into_value(value),
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
            snd_arg: Self::into_value(value),
        }
    }
}

/// A simpler alternative to [`FieldLike`]
///
/// It will convert the `rhs` into a sql value and compare it using the normal
/// like and not like operators.
pub trait SimpleFieldLike<'rhs, Rhs, Any = ()> {
    /// Converts the rhs into a sql value
    fn into_value(rhs: Rhs) -> Value<'rhs>;
}
impl<'rhs, Rhs, Any, T> FieldLike<'rhs, Rhs, private::SimpleFieldLike<Any>> for T
where
    Rhs: 'rhs,
    T: SimpleFieldLike<'rhs, Rhs, Any> + FieldType,
{
    type LiCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_like<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::LiCond<I> {
        Binary {
            operator: BinaryOperator::Like,
            fst_arg: Column(field),
            snd_arg: Self::into_value(value),
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
            snd_arg: Self::into_value(value),
        }
    }
}

/// A simpler alternative to [`FieldIn`]
///
/// It will convert the `rhs`'s items into a sql value and compare it using the normal
/// in and not in operators.
pub trait SimpleFieldIn<'rhs, RhsItem, Any = ()> {
    /// Converts the rhs into a sql value
    fn into_value(rhs_item: RhsItem) -> Value<'rhs>;
}
impl<'rhs, Rhs, Any, T> FieldIn<'rhs, Rhs, private::SimpleFieldIn<Any>> for T
where
    Rhs: IntoIterator,
    Rhs::Item: 'rhs,
    T: SimpleFieldIn<'rhs, Rhs::Item, Any> + FieldType,
{
    type InCond<I: proxy::FieldProxyImpl> = In<Column<I>, Value<'rhs>>;

    fn field_in<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::InCond<I> {
        In {
            operator: InOperator::In,
            fst_arg: Column(field),
            snd_arg: value.into_iter().map(Self::into_value).collect(),
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
            snd_arg: value.into_iter().map(Self::into_value).collect(),
        }
    }
}

/// A simpler alternative to [`FieldILike`]
///
/// It will convert the `rhs` into a sql value and compare it using the normal
/// ilike and not ilike operators.
#[cfg(feature = "postgres-only")]
pub trait SimpleFieldILike<'rhs, Rhs, Any = ()> {
    /// Converts the rhs into a sql value
    fn into_value(rhs: Rhs) -> Value<'rhs>;
}
#[cfg(feature = "postgres-only")]
impl<'rhs, Rhs, Any, T> FieldILike<'rhs, Rhs, private::SimpleFieldILike<Any>> for T
where
    Rhs: 'rhs,
    T: SimpleFieldLike<'rhs, Rhs, Any> + FieldType,
{
    type IliCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_ilike<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::IliCond<I> {
        Binary {
            operator: BinaryOperator::Like,
            fst_arg: Column(field),
            snd_arg: Self::into_value(value),
        }
    }

    type NilCond<I: proxy::FieldProxyImpl> = Binary<Column<I>, Value<'rhs>>;

    fn field_not_ilike<I: proxy::FieldProxyImpl>(
        field: proxy::FieldProxy<I>,
        value: Rhs,
    ) -> Self::NilCond<I> {
        Binary {
            operator: BinaryOperator::NotLike,
            fst_arg: Column(field),
            snd_arg: Self::into_value(value),
        }
    }
}

mod private {
    use std::marker::PhantomData;

    pub struct SimpleFieldEq<T>(PhantomData<T>);
    pub struct SimpleFieldLike<T>(PhantomData<T>);
    pub struct SimpleFieldIn<T>(PhantomData<T>);
    #[cfg(feature = "postgres-only")]
    pub struct SimpleFieldILike<T>(PhantomData<T>);
}
