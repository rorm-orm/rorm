use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;

use crate::conditions::Value;
use crate::const_fn;
use crate::crud::decoder::Decoder;
use crate::fields::proxy;
use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::fields::traits::Columns;
use crate::fields::traits::{FieldColumns, FieldType};
use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::fake_field::FakeField;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::query_context::QueryContext;

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
    /// [`crate::fields::traits::FieldType::GetAnnotations`] implementation for `Option<T>`
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
