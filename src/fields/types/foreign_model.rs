//! The [ForeignModel] field type

use std::fmt;

use rorm_db::Executor;

use crate::conditions::{Binary, BinaryOperator, Column};
use crate::internal::field::{FieldProxy, SingleColumnField};
use crate::model::{GetField, Model, Unrestricted};
use crate::query;

/// Alias for [ForeignModelByField] which only takes a model uses to its primary key.
pub type ForeignModel<M> = ForeignModelByField<<M as Model>::Primary>;

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
pub enum ForeignModelByField<FF: SingleColumnField> {
    /// The other model's primary key which can be used to query it later.
    Key(FF::Type),
    /// The other model's queried instance.
    Instance(Box<FF::Model>),
}

impl<FF: SingleColumnField> ForeignModelByField<FF>
where
    FF::Model: GetField<FF>, // always true
{
    /// Get the instance, if it is available
    pub fn instance(&self) -> Option<&FF::Model> {
        match self {
            Self::Key(_) => None,
            Self::Instance(instance) => Some(instance.as_ref()),
        }
    }

    /// Get the key
    pub fn key(&self) -> &FF::Type {
        match self {
            Self::Key(key) => key,
            Self::Instance(instance) => instance.borrow_field(),
        }
    }

    /// Take the instance, if it is available, or queries it, if not.
    pub async fn take_or_query(self, executor: impl Executor<'_>) -> Result<FF::Model, crate::Error>
    where
        FF::Model: Model<QueryPermission = Unrestricted>,
    {
        match self {
            ForeignModelByField::Key(key) => {
                query!(executor, FF::Model)
                    .condition(Binary {
                        operator: BinaryOperator::Equals,
                        fst_arg: Column(FieldProxy::<FF, FF::Model>::new()),
                        snd_arg: FF::type_into_value(key),
                    })
                    .one()
                    .await
            }
            ForeignModelByField::Instance(instance) => Ok(*instance),
        }
    }
}

impl<FF: SingleColumnField> fmt::Debug for ForeignModelByField<FF>
where
    FF::Model: fmt::Debug,
    FF::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForeignModelByField::Key(key) => f
                .debug_tuple("ForeignModelByField::Key")
                .field(key)
                .finish(),
            ForeignModelByField::Instance(instance) => f
                .debug_tuple("ForeignModelByField::Instance")
                .field(instance)
                .finish(),
        }
    }
}
impl<FF: SingleColumnField> Clone for ForeignModelByField<FF>
where
    FF::Model: Clone,
    FF::Type: Clone,
{
    fn clone(&self) -> Self {
        match self {
            ForeignModelByField::Key(key) => ForeignModelByField::Key(key.clone()),
            ForeignModelByField::Instance(instance) => {
                ForeignModelByField::Instance(instance.clone())
            }
        }
    }
}
