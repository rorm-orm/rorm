//! Various helper for implementing the `FieldType` traits

use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::fields::proxy;
use crate::fields::traits::{FieldEq, FieldType};

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
mod private {
    use std::marker::PhantomData;

    pub struct SimpleFieldEq<T>(PhantomData<T>);
}
