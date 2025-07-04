use crate::conditions::{Binary, BinaryOperator, Column, Value};
use crate::fields::proxy;
use crate::fields::traits::{FieldEq, FieldType};

pub trait SimpleFieldEq<'rhs, Rhs, Any = ()>: FieldType {
    fn into_value(rhs: Rhs) -> Value<'rhs>;
}
impl<'rhs, Rhs, Any, T> FieldEq<'rhs, Rhs, private::SimpleFieldEq<Any>> for T
where
    Rhs: 'rhs,
    T: SimpleFieldEq<'rhs, Rhs, Any>,
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
