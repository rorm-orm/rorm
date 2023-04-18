//! Implementation detail of [`ForeignModelByField`]

use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::fields::ForeignModelByField;
use crate::internal::field::{kind, Field, FieldType, RawField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::model::{GetField, Model};
use crate::sealed;

impl<FF: Field<kind::AsDbType>> FieldType for ForeignModelByField<FF>
where
    FF::Model: GetField<FF>, // always true
{
    type Kind = kind::ForeignModel;

    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [FF::new().into_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.get_field(),
        })]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [FF::new().as_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.borrow_field(),
        })]
    }
}
impl<FF: Field<kind::AsDbType>> FieldType for Option<ForeignModelByField<FF>>
where
    FF::Model: GetField<FF>, // always true
{
    type Kind = kind::ForeignModel;

    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [if let Some(value) = self {
            let [value] = value.into_values();
            value
        } else {
            Value::Null(<FF::DbType as DbType>::NULL_TYPE)
        }]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [if let Some(value) = self {
            let [value] = value.as_values();
            value
        } else {
            Value::Null(<FF::DbType as DbType>::NULL_TYPE)
        }]
    }
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

impl<FF: Field<kind::AsDbType>> ForeignModelTrait for ForeignModelByField<FF>
where
    FF::Model: GetField<FF>, // always true
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
    FF::Model: GetField<FF>, // always true
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
