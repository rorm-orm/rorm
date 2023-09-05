//! Implementation detail of [`ForeignModelByField`]

use std::marker::PhantomData;

use rorm_db::row::DecodeOwned;
use rorm_db::{Error, Row};

use crate::conditions::Value;
use crate::crud::decoder::Decoder;
use crate::fields::types::ForeignModelByField;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::{kind, Field, FieldProxy, FieldType, RawField, SingleColumnField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::internal::hmr::Source;
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;
use crate::model::{GetField, Model};
use crate::{impl_FieldEq, sealed};

impl<FF> FieldType for ForeignModelByField<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    FF::Model: GetField<FF>, // always true
    FF: SingleColumnField,
{
    type Kind = kind::ForeignModel;

    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [FF::type_into_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.get_field(),
        })]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [FF::type_as_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.borrow_field(),
        })]
    }

    type Decoder = ForeignModelByFieldDecoder<FF>;
}

impl<FF> FieldType for Option<ForeignModelByField<FF>>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    FF::Model: GetField<FF>, // always true
    FF: SingleColumnField,
    Option<FF::Type>: AsDbType,
{
    type Kind = kind::ForeignModel;

    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        self.map(ForeignModelByField::into_values)
            .unwrap_or([Value::Null(
                <<Option<FF::Type> as AsDbType>::DbType>::NULL_TYPE,
            )])
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        self.as_ref()
            .map(ForeignModelByField::as_values)
            .unwrap_or([Value::Null(
                <<Option<FF::Type> as AsDbType>::DbType>::NULL_TYPE,
            )])
    }

    type Decoder = OptionForeignModelByFieldDecoder<FF>;
}

#[doc(hidden)]
pub trait ForeignModelTrait: FieldType<Kind = kind::ForeignModel> {
    sealed!(trait);

    type RelatedField: Field<kind::AsDbType>;
    type Primitive: DecodeOwned;
    const IS_OPTION: bool;

    fn from_primitive(primitive: Self::Primitive) -> Self;
    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type>;
}

impl<FF> ForeignModelTrait for ForeignModelByField<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    FF::Model: GetField<FF>, // always true
    FF: SingleColumnField,
{
    sealed!(impl);

    type RelatedField = FF;
    type Primitive = FF::Primitive;
    const IS_OPTION: bool = false;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        ForeignModelByField::Key(FF::new().from_primitive(primitive))
    }

    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        Some(match self {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.borrow_field(),
        })
    }
}

impl<FF: Field<kind::AsDbType>> ForeignModelTrait for Option<ForeignModelByField<FF>>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    FF::Model: GetField<FF>, // always true
    FF: SingleColumnField,
    Option<FF::Type>: AsDbType,
{
    sealed!(impl);

    type RelatedField = FF;
    type Primitive = Option<FF::Primitive>;
    const IS_OPTION: bool = true;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        primitive.map(|primitive| {
            ForeignModelByField::Key(Self::RelatedField::new().from_primitive(primitive))
        })
    }

    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        self.as_ref().map(|value| match value {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.borrow_field(),
        })
    }
}

pub(crate) type RF<F> = <<F as RawField>::Type as ForeignModelTrait>::RelatedField;
impl<F> Field<kind::ForeignModel> for F
where
    F: RawField<Kind = kind::ForeignModel>,
    F::Type: ForeignModelTrait,
    <<F::Type as ForeignModelTrait>::RelatedField as RawField>::Model:
        GetField<<F::Type as ForeignModelTrait>::RelatedField>, // always true
{
    sealed!(impl);

    type DbType = <RF<F> as Field<kind::AsDbType>>::DbType;

    const ANNOTATIONS: Annotations = {
        let mut annos = Self::EXPLICIT_ANNOTATIONS;
        annos.nullable = <<F as RawField>::Type as ForeignModelTrait>::IS_OPTION;
        if annos.max_length.is_none() {
            annos.max_length = <RF<F> as Field<kind::AsDbType>>::ANNOTATIONS.max_length;
        }
        annos.foreign = Some(hmr::annotations::ForeignKey {
            table_name: <RF<F> as RawField>::Model::TABLE,
            column_name: <RF<F> as RawField>::NAME,
        });
        annos
    };
    type Primitive = <<F as RawField>::Type as ForeignModelTrait>::Primitive;

    fn from_primitive(self, primitive: Self::Primitive) -> Self::Type {
        <<F as RawField>::Type as ForeignModelTrait>::from_primitive(primitive)
    }
}

/// [`FieldDecoder`] for [`ForeignModelByField<FF>`]
pub struct ForeignModelByFieldDecoder<FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>>(
    <FF::Type as FieldType>::Decoder,
);
impl<FF> Decoder for ForeignModelByFieldDecoder<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
{
    type Result = ForeignModelByField<FF>;

    fn by_name(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0.by_name(row).map(ForeignModelByField::Key)
    }

    fn by_index(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0.by_index(row).map(ForeignModelByField::Key)
    }
}
impl<FF> FieldDecoder for ForeignModelByFieldDecoder<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
{
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        Self(FieldDecoder::new(
            ctx,
            FieldProxy::<FakeFieldType<FF::Type, F>, P>::new(),
        ))
    }
}

/// [`FieldDecoder`] for [`Option<ForeignModelByField<FF>>`](ForeignModelByField)
pub struct OptionForeignModelByFieldDecoder<
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
>(<Option<FF::Type> as FieldType>::Decoder)
where
    Option<FF::Type>: FieldType;
impl<FF> Decoder for OptionForeignModelByFieldDecoder<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    Option<FF::Type>: FieldType,
{
    type Result = Option<ForeignModelByField<FF>>;

    fn by_name(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0
            .by_name(row)
            .map(|option| option.map(ForeignModelByField::Key))
    }

    fn by_index(&self, row: &Row) -> Result<Self::Result, Error> {
        self.0
            .by_index(row)
            .map(|option| option.map(ForeignModelByField::Key))
    }
}
impl<FF> FieldDecoder for OptionForeignModelByFieldDecoder<FF>
where
    FF: RawField<Kind = kind::AsDbType> + Field<kind::AsDbType>,
    Option<FF::Type>: FieldType,
{
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        Self(FieldDecoder::new(
            ctx,
            FieldProxy::<FakeFieldType<Option<FF::Type>, F>, P>::new(),
        ))
    }
}

/// Take a field `F` and create a new "fake" field with the different [`RawField::Type`](RawField::Type) `T`
#[allow(non_camel_case_types)]
struct FakeFieldType<T, F>(PhantomData<(T, F)>);
impl<T, F> Clone for FakeFieldType<T, F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, F> Copy for FakeFieldType<T, F> {}
impl<T, F> RawField for FakeFieldType<T, F>
where
    T: FieldType + 'static,
    F: RawField,
{
    type Kind = T::Kind;
    type Type = T;
    type Model = F::Model;
    const INDEX: usize = F::INDEX;
    const NAME: &'static str = F::NAME;
    const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
    const SOURCE: Option<Source> = F::SOURCE;
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, FF::Type, FieldEq_ForeignModelByField_Owned> for ForeignModelByField<FF>
    where
        FF: RawField<Kind = kind::AsDbType>,
        FF: Field<kind::AsDbType>,
        FF: SingleColumnField,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_into_value }
);
impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, FF::Type, FieldEq_ForeignModelByField_Owned> for Option<ForeignModelByField<FF>>
    where
        FF: RawField<Kind = kind::AsDbType>,
        FF: Field<kind::AsDbType>,
        FF: SingleColumnField,
        Option<FF::Type>: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_into_value }
);

impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, &'rhs FF::Type, FieldEq_ForeignModelByField_Borrowed> for ForeignModelByField<FF>
    where
        FF: RawField<Kind = kind::AsDbType>,
        FF: Field<kind::AsDbType>,
        FF: SingleColumnField,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_as_value }
);
impl_FieldEq!(
    impl<'rhs, FF> FieldEq<'rhs, &'rhs FF::Type, FieldEq_ForeignModelByField_Borrowed> for Option<ForeignModelByField<FF>>
    where
        FF: RawField<Kind = kind::AsDbType>,
        FF: Field<kind::AsDbType>,
        FF: SingleColumnField,
        Option<FF::Type>: AsDbType,
        FF::Model: GetField<FF>, // always true
    { <FF as SingleColumnField>::type_as_value }
);

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Owned;
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct FieldEq_ForeignModelByField_Borrowed;
