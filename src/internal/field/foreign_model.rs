//! Implementation detail of [`ForeignModelByField`]

use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::fields::ForeignModelByField;
use crate::internal::field::{kind, Field, FieldType, RawField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::model::{GetField, Model};

impl<FF: Field<kind::AsDbType>> FieldType for ForeignModelByField<FF> {
    type Kind = kind::ForeignModel;
}
impl<FF: Field<kind::AsDbType>> FieldType for Option<ForeignModelByField<FF>> {
    type Kind = kind::ForeignModel;
}

#[doc(hidden)]
pub trait ForeignModelTrait: FieldType<Kind = kind::ForeignModel> {
    type RelatedField: Field<kind::AsDbType>;
    type Primitive: DecodeOwned;
    const IS_OPTION: bool;

    fn from_primitive(primitive: Self::Primitive) -> Self;
    fn as_condition_value(&self) -> Value;
    fn into_condition_value(self) -> Value<'static>;
    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type>;
}

impl<FF: Field<kind::AsDbType>> ForeignModelTrait for ForeignModelByField<FF>
where
    FF::Model: GetField<FF>, // always true
{
    type RelatedField = FF;
    type Primitive = FF::Primitive;
    const IS_OPTION: bool = false;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        ForeignModelByField::Key(FF::from_primitive(primitive))
    }

    fn as_condition_value(&self) -> Value {
        FF::as_condition_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.borrow_field(),
        })
    }

    fn into_condition_value(self) -> Value<'static> {
        FF::into_condition_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.get_field(),
        })
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
    type RelatedField = FF;
    type Primitive = Option<FF::Primitive>;
    const IS_OPTION: bool = true;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        primitive.map(|primitive| {
            ForeignModelByField::Key(Self::RelatedField::from_primitive(primitive))
        })
    }

    fn as_condition_value(&self) -> Value {
        if let Some(value) = self {
            Self::RelatedField::as_condition_value(match value {
                ForeignModelByField::Key(value) => value,
                ForeignModelByField::Instance(model) => model.borrow_field(),
            })
        } else {
            Value::Null(<FF::DbType as DbType>::NULL_TYPE)
        }
    }

    fn into_condition_value(self) -> Value<'static> {
        if let Some(value) = self {
            Self::RelatedField::into_condition_value(match value {
                ForeignModelByField::Key(value) => value,
                ForeignModelByField::Instance(model) => model.get_field(),
            })
        } else {
            Value::Null(<FF::DbType as DbType>::NULL_TYPE)
        }
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

    fn from_primitive(primitive: Self::Primitive) -> Self::Type {
        <<F as RawField>::Type as ForeignModelTrait>::from_primitive(primitive)
    }

    fn as_condition_value(value: &Self::Type) -> Value {
        <<F as RawField>::Type as ForeignModelTrait>::as_condition_value(value)
    }

    fn into_condition_value(value: Self::Type) -> Value<'static> {
        <<F as RawField>::Type as ForeignModelTrait>::into_condition_value(value)
    }
}
