//! Implementation detail of [`ForeignModelByField`]

use std::marker::PhantomData;

use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;

use crate::conditions::Value;
use crate::const_fn;
use crate::crud::decoder::Decoder;
use crate::fields::proxy;
use crate::fields::proxy::FieldProxyImpl;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldLike};
#[cfg(feature = "postgres-only")]
use crate::fields::traits::FieldILike;
use crate::fields::traits::{Array, FieldColumns};
use crate::fields::types::ForeignModelByField;
use crate::fields::utils::get_names::single_column_name;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::fake_field::FakeField;
use crate::internal::field::{Field, FieldProxy, FieldType, SingleColumnField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::query_context::QueryContext;
use crate::model::Model;
use crate::sealed;

impl<FF> FieldType for ForeignModelByField<FF>
where
    FF: SingleColumnField,
{
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = FF::Type::NULL;

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [FF::type_into_value(self.0)]
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [FF::type_as_value(&self.0)]
    }

    type Decoder = ForeignModelByFieldDecoder<FF>;

    type GetAnnotations = foreign_annotations<FF>;

    type Check = <FF::Type as FieldType>::Check;

    type GetNames = single_column_name;
}

#[doc(hidden)]
pub trait ForeignModelTrait {
    sealed!(trait);

    type RelatedField: SingleColumnField;
    fn as_key(&self) -> Option<&<Self::RelatedField as Field>::Type>;
}

impl<FF> ForeignModelTrait for ForeignModelByField<FF>
where
    FF: SingleColumnField,
{
    sealed!(impl);

    type RelatedField = FF;
    fn as_key(&self) -> Option<&<Self::RelatedField as Field>::Type> {
        Some(&self.0)
    }
}

impl<FF: SingleColumnField> ForeignModelTrait for Option<ForeignModelByField<FF>>
where
    FF: SingleColumnField,
{
    sealed!(impl);

    type RelatedField = FF;

    fn as_key(&self) -> Option<&<Self::RelatedField as Field>::Type> {
        self.as_ref().map(|value| &value.0)
    }
}

const_fn! {
    /// - sets `foreign`
    pub fn foreign_annotations<FF: SingleColumnField>(field: Annotations) -> [Annotations; 1] {
        let mut annos = field;
        annos.foreign = Some(hmr::annotations::ForeignKey {
            table_name: FF::Model::TABLE,
            column_name: &FF::NAME,
        });
        [annos]
    }
}

/// Trait alias for a `Field` which points to a foreign model
pub trait ForeignModelField: SingleColumnField<Type: ForeignModelTrait> {}

pub(crate) type RF<F> = <<F as Field>::Type as ForeignModelTrait>::RelatedField;
impl<F> ForeignModelField for F where F: SingleColumnField<Type: ForeignModelTrait> {}

/// [`FieldDecoder`] for [`ForeignModelByField<FF>`]
pub struct ForeignModelByFieldDecoder<FF: SingleColumnField>(<FF::Type as FieldType>::Decoder);
impl<FF: SingleColumnField> Decoder for ForeignModelByFieldDecoder<FF> {
    type Result = ForeignModelByField<FF>;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_name(row).map(ForeignModelByField)
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_index(row).map(ForeignModelByField)
    }
}
impl<FF> FieldDecoder for ForeignModelByFieldDecoder<FF>
where
    FF: SingleColumnField,
{
    fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>,
    {
        Self(FieldDecoder::new(
            ctx,
            proxy::new::<(FakeField<FF::Type, I::Field>, I::Path)>(),
        ))
    }
}

impl<FF, Rhs> SimpleFieldEq<Rhs> for ForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldEq<Rhs>,
{
}
impl<FF, Rhs> SimpleFieldIn<Rhs> for ForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldIn<Rhs>,
{
}
impl<FF, Rhs> SimpleFieldLike<Rhs> for ForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldLike<Rhs>,
{
}

#[cfg(feature = "postgres-only")]
impl<'rhs, Rhs, Any, FF> FieldILike<'rhs, Rhs, FieldLike_ForeignModelByField<Any>>
    for ForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: FieldILike<'rhs, Rhs, Any> + FieldType<Columns = Array<1>>,
{
    type IliCond<I: FieldProxyImpl> = <FF::Type as FieldILike<'rhs, Rhs, Any>>::IliCond<I>;

    fn field_ilike<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::IliCond<I> {
        <FF::Type as FieldILike<'rhs, Rhs, Any>>::field_ilike(field, value)
    }

    type NilCond<I: FieldProxyImpl> = <FF::Type as FieldILike<'rhs, Rhs, Any>>::NilCond<I>;

    fn field_not_ilike<I: FieldProxyImpl>(field: FieldProxy<I>, value: Rhs) -> Self::NilCond<I> {
        <FF::Type as FieldILike<'rhs, Rhs, Any>>::field_not_ilike(field, value)
    }
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Borrowed;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldIn_ForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldIn_ForeignModelByField_Borrowed;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldLike_ForeignModelByField<Any>(PhantomData<Any>);
#[doc(hidden)]
#[allow(non_camel_case_types)]
#[cfg(feature = "postgres-only")]
pub struct FieldILike_ForeignModelByField<Any>(PhantomData<Any>);
