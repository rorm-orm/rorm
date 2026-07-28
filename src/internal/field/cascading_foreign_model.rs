//! Implementation detail of [`CascadingForeignModelByField`]

use std::marker::PhantomData;

use generic_array::typenum::U1;
use generic_array::{arr, GenericArray};
use rorm_db::row::RowError;
use rorm_db::sql::value::NullType;
use rorm_db::Row;

use crate::conditions::Value;
use crate::const_fn;
use crate::crud::decoder::Decoder;
use crate::fields::proxy;
use crate::fields::proxy::FieldProxyImpl;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldLike};
use crate::fields::traits::FieldColumns;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::FieldILike;
use crate::fields::types::CascadingForeignModelByField;
use crate::fields::utils::get_names::single_column_name;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::fake_field::FakeField;
use crate::internal::field::foreign_model::ForeignModelTrait;
use crate::internal::field::{Field, FieldProxy, FieldType, SingleColumnField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::{Annotations, OnDelete, OnUpdate};
use crate::internal::query_context::QueryContext;
use crate::model::Model;
use crate::sealed;

impl<FF> FieldType for CascadingForeignModelByField<FF>
where
    FF: SingleColumnField,
{
    type Columns = U1;

    const NULL: FieldColumns<Self, NullType> = FF::Type::NULL;

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        arr![FF::type_into_value(self.0)]
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        arr![FF::type_as_value(&self.0)]
    }

    type Decoder = CascadingForeignModelByFieldDecoder<FF>;

    type GetAnnotations = cascading_foreign_annotations<FF>;

    type Check = <FF::Type as FieldType>::Check;

    type GetNames = single_column_name;
}

impl<FF> ForeignModelTrait for CascadingForeignModelByField<FF>
where
    FF: SingleColumnField,
{
    sealed!(impl);

    type RelatedField = FF;
    fn as_key(&self) -> Option<&<Self::RelatedField as Field>::Type> {
        Some(&self.0)
    }
}

impl<FF: SingleColumnField> ForeignModelTrait for Option<CascadingForeignModelByField<FF>>
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
    /// - copies `max_length` from the foreign key
    /// - sets `foreign`
    /// - sets `on_update` and `on_delete` to `Cascade`
    pub fn cascading_foreign_annotations<FF: SingleColumnField>(field: Annotations) -> GenericArray<Annotations, U1> {
        let mut annos = field;
        if annos.max_length.is_none() {
            let target_annos = FF::EFFECTIVE_ANNOTATION;
            annos.max_length = target_annos.max_length;
        }
        annos.foreign = Some(hmr::annotations::ForeignKey {
            table_name: FF::Model::TABLE,
            column_name: &FF::NAME,
        });
        if field.on_update.is_some() || field.on_delete.is_some() {
            panic!("The annotations `on_update` and `on_delete` are implied by their field's type and can't be set explicitly");
        }
        annos.on_update = Some(OnUpdate::Cascade);
        annos.on_delete = Some(OnDelete::Cascade);
        arr![annos]
    }
}

/// [`FieldDecoder`] for [`CascadingForeignModelByField<FF>`]
pub struct CascadingForeignModelByFieldDecoder<FF: SingleColumnField>(
    <FF::Type as FieldType>::Decoder,
);
impl<FF: SingleColumnField> Decoder for CascadingForeignModelByFieldDecoder<FF> {
    type Result = CascadingForeignModelByField<FF>;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_name(row).map(CascadingForeignModelByField)
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.0.by_index(row).map(CascadingForeignModelByField)
    }
}
impl<FF> FieldDecoder for CascadingForeignModelByFieldDecoder<FF>
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

impl<FF, Rhs> SimpleFieldEq<Rhs> for CascadingForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldEq<Rhs>,
{
}
impl<FF, Rhs> SimpleFieldIn<Rhs> for CascadingForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldIn<Rhs>,
{
}
impl<FF, Rhs> SimpleFieldLike<Rhs> for CascadingForeignModelByField<FF>
where
    FF: SingleColumnField,
    FF::Type: SimpleFieldLike<Rhs>,
{
}

#[cfg(feature = "postgres-only")]
impl<'rhs, Rhs, Any, FF> FieldILike<'rhs, Rhs, FieldLike_CascadingForeignModelByField<Any>>
    for CascadingForeignModelByField<FF>
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
pub struct FieldEq_CascadingForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_CascadingForeignModelByField_Borrowed;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldIn_CascadingForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldIn_CascadingForeignModelByField_Borrowed;

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldLike_CascadingForeignModelByField<Any>(PhantomData<Any>);
#[doc(hidden)]
#[allow(non_camel_case_types)]
#[cfg(feature = "postgres-only")]
pub struct FieldILike_CascadingForeignModelByField<Any>(PhantomData<Any>);
