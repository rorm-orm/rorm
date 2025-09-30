use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;

use crate::conditions::{Column, Condition, Unary, UnaryOperator, Value};
use crate::crud::decoder::Decoder;
use crate::fields::proxy;
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::fields::traits::{Array, Columns, FieldEq, FieldLike};
use crate::fields::traits::{FieldColumns, FieldType};
use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::fake_field::FakeField;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::query_context::{ConditionBuilder, QueryContext};
use crate::{and, const_fn, or};

impl<'rhs, T, Rhs: 'rhs, Any> FieldEq<'rhs, Option<Rhs>, private::OptionFieldEq<Any>> for Option<T>
where
    T: FieldType<Columns = Array<1>>,
    T: FieldEq<'rhs, Rhs, Any>,
{
    type EqCond<I: FieldProxyImpl> =
        OptionFieldEqCond<I, <T as FieldEq<'rhs, Rhs, Any>>::EqCond<I>>;

    fn field_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: Option<Rhs>,
    ) -> Self::EqCond<I> {
        OptionFieldEqCond {
            not: false,
            // where clause required `T` to be a single column field
            column: field,
            value: value.map(|value| <T as FieldEq<'rhs, Rhs, Any>>::field_equals(field, value)),
        }
    }

    type NeCond<I: FieldProxyImpl> =
        OptionFieldEqCond<I, <T as FieldEq<'rhs, Rhs, Any>>::NeCond<I>>;

    fn field_not_equals<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: Option<Rhs>,
    ) -> Self::NeCond<I> {
        OptionFieldEqCond {
            not: true,
            // where clause required `T` to be a single column field
            column: field,
            value: value
                .map(|value| <T as FieldEq<'rhs, Rhs, Any>>::field_not_equals(field, value)),
        }
    }
}

impl<'rhs, T, Rhs: 'rhs, Any> FieldLike<'rhs, Option<Rhs>, private::OptionFieldLike<Any>>
    for Option<T>
where
    T: FieldType<Columns = Array<1>>,
    T: FieldLike<'rhs, Rhs, Any>,
{
    type LiCond<I: FieldProxyImpl> =
        OptionFieldEqCond<I, <T as FieldLike<'rhs, Rhs, Any>>::LiCond<I>>;

    fn field_like<I: FieldProxyImpl>(field: FieldProxy<I>, value: Option<Rhs>) -> Self::LiCond<I> {
        OptionFieldEqCond {
            not: false,
            // where clause required `T` to be a single column field
            column: field,
            value: value.map(|value| <T as FieldLike<'rhs, Rhs, Any>>::field_like(field, value)),
        }
    }

    type NlCond<I: FieldProxyImpl> =
        OptionFieldEqCond<I, <T as FieldLike<'rhs, Rhs, Any>>::NlCond<I>>;

    fn field_not_like<I: FieldProxyImpl>(
        field: FieldProxy<I>,
        value: Option<Rhs>,
    ) -> Self::NlCond<I> {
        OptionFieldEqCond {
            not: true,
            // where clause required `T` to be a single column field
            column: field,
            value: value
                .map(|value| <T as FieldLike<'rhs, Rhs, Any>>::field_not_like(field, value)),
        }
    }
}

/// Condition produced by `Option<T>`'s [`FieldEq`] (and c[`FieldLike`]) implementation
///
/// It wraps an optional condition produced by `T`'s `FieldEq` implementation.
///
/// On `None` it will emit a `IS NULL` (or `IS NOT NULL`) check.
/// On `Some` it will use `T`'s condition after checking for `NULL` explicitly
/// instead of hoping `T` to do the right thing with `NULL`.
pub struct OptionFieldEqCond<I, C> {
    /// Is the equality check negated?
    not: bool,

    /// The column to check
    ///
    /// This MUST be a column i.e. a single column field.
    /// This is enforced by an extra where bound in `Option<T>`'s implementation.
    column: FieldProxy<I>,

    /// The condition produced by `T` or `None` if the right hand side was `None`.
    value: Option<C>,
}
impl<'a, I, C> Condition<'a> for OptionFieldEqCond<I, C>
where
    I: FieldProxyImpl,
    C: Condition<'a>,
{
    fn build(&self, builder: ConditionBuilder<'_, 'a>) {
        if !self.not {
            // equals
            match &self.value {
                None => Unary {
                    operator: UnaryOperator::IsNull,
                    fst_arg: Column(self.column),
                }
                .build(builder),
                Some(condition) => and![
                    Unary {
                        operator: UnaryOperator::IsNotNull,
                        fst_arg: Column(self.column),
                    },
                    condition
                ]
                .build(builder),
            }
        } else {
            // no equals
            match &self.value {
                None => Unary {
                    operator: UnaryOperator::IsNotNull,
                    fst_arg: Column(self.column),
                }
                .build(builder),
                Some(condition) => or![
                    Unary {
                        operator: UnaryOperator::IsNull,
                        fst_arg: Column(self.column),
                    },
                    condition
                ]
                .build(builder),
            }
        }
    }
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
    type GetAnnotations = get_option_annotations<T>;
    type Check = T::Check;
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

const_fn! {
    /// [`FieldType::GetAnnotations`] implementation for `Option<T>`
    pub fn get_option_annotations<T: FieldType>(#[raw] Arg: (Annotations,)) -> FieldColumns<T, Annotations> {
        type CallInner<T, Arg> = <<T as FieldType>::GetAnnotations as ConstFn<
            (Annotations,),
            FieldColumns<T, Annotations>,
        >>::Body<Arg>;
        type CallOuter<T, Arg> = <<<T as FieldType>::Columns as Columns>::SetNull as ConstFn<
            (FieldColumns<T, Annotations>,),
            FieldColumns<T, Annotations>,
        >>::Body<Arg>;
        <CallOuter<T, (CallInner<T, Arg>,)> as Contains<_>>::ITEM
    }
}

mod private {
    use std::marker::PhantomData;

    pub struct OptionFieldEq<T>(PhantomData<T>);
    pub struct OptionFieldLike<T>(PhantomData<T>);
}
