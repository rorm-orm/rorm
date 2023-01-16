//! The [ForeignModel] field type

use rorm_declaration::imr;

use crate::conditions::Value;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{kind, Field, FieldType, OptionField, RawField};
use crate::internal::hmr::annotations::Annotations;
use crate::model::Model;
use crate::value::NullType;

/// Alias for [ForeignModelByField] which defaults the second generic parameter to use the primary key.
///
/// This default is only provided on this alias instead of the enum itself,
/// because this way internal code must provide the parameter while users can ignore it.
/// Forgetting to set it in internal code could lead to some bugs which are nasty to find.
pub type ForeignModel<M, T = <<M as Model>::Primary as Field>::Type> = ForeignModelByField<M, T>;

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
#[derive(Clone, Debug)]
pub enum ForeignModelByField<M: Model, T> {
    /// The other model's primary key which can be used to query it later.
    Key(T),
    /// The other model's queried instance.
    Instance(Box<M>),
}
impl<M: Model, T> ForeignModelByField<M, T> {
    /// Get the instance, if it is available
    pub fn instance(&self) -> Option<&M> {
        match self {
            Self::Key(_) => None,
            Self::Instance(instance) => Some(instance.as_ref()),
        }
    }
}
impl<M: Model, T> From<T> for ForeignModelByField<M, T> {
    fn from(key: T) -> Self {
        Self::Key(key)
    }
}

pub(crate) type RelatedField<M, F> =
    <<F as RawField>::RelatedField as OptionField>::UnwrapOr<<M as Model>::Primary>;

impl<M: Model, T: AsDbType> FieldType for ForeignModelByField<M, T> {
    type Kind = kind::AsDbType;
}
impl<M: Model, T: AsDbType> AsDbType for ForeignModelByField<M, T> {
    type Primitive = T::Primitive;
    type DbType<F: Field> =
        <<RelatedField<M, F> as Field>::Type as AsDbType>::DbType<RelatedField<M, F>>;

    const NULL_TYPE: NullType = T::NULL_TYPE;

    const IMPLICIT: Option<Annotations> = {
        let mut annos = if let Some(annos) = T::IMPLICIT {
            annos
        } else {
            Annotations::empty()
        };
        annos.foreign = true;
        Some(annos)
    };

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Self::Key(T::from_primitive(primitive))
    }

    fn as_primitive<F: Field>(&self) -> Value {
        match self {
            ForeignModelByField::Key(value) => value.as_primitive::<F>(),
            ForeignModelByField::Instance(model) => {
                if let Some(value) = model.get_value(RelatedField::<M, F>::INDEX) {
                    value
                } else {
                    unreachable!("A model should contain its primary key");
                }
            }
        }
    }

    fn custom_annotations<F: Field>(annotations: &mut Vec<imr::Annotation>) {
        let related_annotations = RelatedField::<M, F>::ANNOTATIONS;
        if let Some(max_length) = related_annotations.max_length {
            if !annotations
                .iter()
                .any(|anno| matches!(anno, imr::Annotation::MaxLength(_)))
            {
                annotations.push(imr::Annotation::MaxLength(max_length.0));
            }
        }
        annotations.push(imr::Annotation::ForeignKey(imr::ForeignKey {
            table_name: M::TABLE.to_string(),
            column_name: RelatedField::<M, F>::NAME.to_string(),
            on_delete: F::ANNOTATIONS.on_delete.unwrap_or_default(),
            on_update: F::ANNOTATIONS.on_update.unwrap_or_default(),
        }))
    }
}
