//! Implementation detail of [`ForeignModelByField`]

use rorm_db::row::DecodeOwned;

use crate::conditions::Value;
use crate::fields::ForeignModelByField;
use crate::internal::field::{kind, Field, FieldType, OptionField, RawField};
use crate::internal::hmr;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::model::{GetField, Model};

/// Shorthand to access the related field to a [`ForeignModel`]
pub type RelatedField<F> = <<F as RawField>::Type as ForeignModelTrait<F>>::RelatedField;

impl<M: Model, T> FieldType for ForeignModelByField<M, T> {
    type Kind = kind::ForeignModel;
}

impl<M: Model, T> FieldType for Option<ForeignModelByField<M, T>> {
    type Kind = kind::ForeignModel;
}

#[doc(hidden)]
pub trait ForeignModelTrait<F>: FieldType<Kind = kind::ForeignModel>
where
    F: RawField<Type = Self, Kind = kind::ForeignModel>,
{
    type RelatedField: Field;
    type Primitive: DecodeOwned;
    const IS_OPTION: bool;

    fn from_primitive(primitive: Self::Primitive) -> Self;
    fn as_condition_value(&self) -> Value;
    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type>;
}

type RelatedFieldUsingModel<M, F> =
    <<F as RawField>::RelatedField as OptionField>::UnwrapOr<<M as Model>::Primary>;

impl<M, F> ForeignModelTrait<F>
    for ForeignModelByField<M, <RelatedFieldUsingModel<M, F> as RawField>::Type>
where
    M: Model,
    RelatedFieldUsingModel<M, F>: Field,
    F: RawField<Type = Self, Kind = kind::ForeignModel>,
    M: GetField<RelatedFieldUsingModel<M, F>>, // Always true
{
    type RelatedField = RelatedFieldUsingModel<M, F>;
    type Primitive = <Self::RelatedField as Field>::Primitive;
    const IS_OPTION: bool = false;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        ForeignModelByField::Key(
            <F::Type as ForeignModelTrait<F>>::RelatedField::from_primitive(primitive),
        )
    }

    fn as_condition_value(&self) -> Value {
        <F::Type as ForeignModelTrait<F>>::RelatedField::as_condition_value(match self {
            ForeignModelByField::Key(value) => value,
            ForeignModelByField::Instance(model) => model.get_field(),
        })
    }

    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        Some(match self {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.get_field(),
        })
    }
}

impl<M, F> ForeignModelTrait<F>
    for Option<ForeignModelByField<M, <RelatedFieldUsingModel<M, F> as RawField>::Type>>
where
    M: Model,
    RelatedFieldUsingModel<M, F>: Field,
    F: RawField<Type = Self, Kind = kind::ForeignModel>,
    M: GetField<RelatedFieldUsingModel<M, F>>, // Always true
{
    type RelatedField = RelatedFieldUsingModel<M, F>;
    type Primitive = Option<<Self::RelatedField as Field>::Primitive>;
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
                ForeignModelByField::Instance(model) => model.get_field(),
            })
        } else {
            Value::Null(<<Self::RelatedField as Field>::DbType as DbType>::NULL_TYPE)
        }
    }

    fn as_key(&self) -> Option<&<Self::RelatedField as RawField>::Type> {
        self.as_ref().map(|value| match value {
            ForeignModelByField::Key(key) => key,
            ForeignModelByField::Instance(instance) => instance.get_field(),
        })
    }
}

impl<F> Field<kind::ForeignModel> for F
where
    F: RawField<Kind = kind::ForeignModel>,
    F::Type: ForeignModelTrait<F>,
{
    type DbType = <RelatedField<F> as Field>::DbType;

    const ANNOTATIONS: Annotations = {
        let mut annos = Self::EXPLICIT_ANNOTATIONS;
        if annos.max_length.is_none() {
            annos.max_length =
                <F::Type as ForeignModelTrait<F>>::RelatedField::ANNOTATIONS.max_length;
        }
        annos.nullable |= <<F as RawField>::Type as ForeignModelTrait<F>>::IS_OPTION;
        annos.foreign = Some(hmr::annotations::ForeignKey {
            table_name: <RelatedField<F> as RawField>::Model::TABLE,
            column_name: <RelatedField<F> as RawField>::NAME,
        });
        annos
    };

    type Primitive = <<F as RawField>::Type as ForeignModelTrait<F>>::Primitive;

    fn from_primitive(primitive: Self::Primitive) -> Self::Type {
        <<F as RawField>::Type as ForeignModelTrait<F>>::from_primitive(primitive)
    }

    fn as_condition_value(value: &Self::Type) -> Value {
        <<F as RawField>::Type as ForeignModelTrait<F>>::as_condition_value(value)
    }
}
