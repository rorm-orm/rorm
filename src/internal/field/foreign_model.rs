//! The [ForeignModel] field type

use crate::conditions::Value;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{Field, OptionField, RawField};
use crate::internal::hmr::annotations::Annotations;
use crate::Model;
use rorm_declaration::imr;

/// Alias for [ForeignModelByField] which defaults the second generic parameter to use the primary key.
///
/// This default is only provided on this alias instead of the enum itself,
/// because this way internal code must provide the parameter while users can ignore it.
/// Forgetting to set it in internal code could lead to some bugs which are nasty to find.
pub type ForeignModel<M, T = <<M as Model>::Primary as Field>::Type> = ForeignModelByField<M, T>;

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
pub enum ForeignModelByField<M: Model, T> {
    /// The other model's primary key which can be used to query it later.
    Key(T),
    /// The other model's queried instance.
    Instance(Box<M>),
}
impl<M: Model, T: AsDbType> Clone for ForeignModelByField<M, T>
where
    M: Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            ForeignModelByField::Key(primary) => ForeignModelByField::Key(primary.clone()),
            ForeignModelByField::Instance(model) => ForeignModelByField::Instance(model.clone()),
        }
    }
}

pub(crate) type RelatedField<M, F> =
    <<F as RawField>::RelatedField as OptionField>::UnwrapOr<<M as Model>::Primary>;

impl<M: Model, T: AsDbType> AsDbType for ForeignModelByField<M, T> {
    type Primitive = T::Primitive;
    type DbType<F: Field> =
        <<RelatedField<M, F> as Field>::Type as AsDbType>::DbType<RelatedField<M, F>>;

    const IMPLICIT: Option<Annotations> = T::IMPLICIT;

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

    const IS_NULLABLE: bool = <T as AsDbType>::IS_NULLABLE;

    fn custom_annotations<F: Field>(annotations: &mut Vec<imr::Annotation>) {
        annotations.push(imr::Annotation::ForeignKey(imr::ForeignKey {
            table_name: M::TABLE.to_string(),
            column_name: RelatedField::<M, F>::NAME.to_string(),
            on_delete: F::ANNOTATIONS.on_delete.unwrap_or_default(),
            on_update: F::ANNOTATIONS.on_update.unwrap_or_default(),
        }))
    }

    const IS_FOREIGN: bool = true;
}
