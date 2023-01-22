//! The [ForeignModel] field type

use crate::internal::field::{kind, Field};
use crate::model::{GetField, Model};

/// Alias for [ForeignModelByField] which only takes a model uses to its primary key.
pub type ForeignModel<M> = ForeignModelByField<<M as Model>::Primary>;

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
#[derive(Clone, Debug)]
pub enum ForeignModelByField<FF: Field<kind::AsDbType>> {
    /// The other model's primary key which can be used to query it later.
    Key(FF::Type),
    /// The other model's queried instance.
    Instance(Box<FF::Model>),
}

impl<FF: Field<kind::AsDbType>> ForeignModelByField<FF>
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
            Self::Instance(instance) => instance.get_field(),
        }
    }
}
