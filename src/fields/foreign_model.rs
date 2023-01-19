//! The [ForeignModel] field type

use crate::internal::field::RawField;
use crate::model::Model;

/// Alias for [ForeignModelByField] which defaults the second generic parameter to use the primary key.
///
/// This default is only provided on this alias instead of the enum itself,
/// because this way internal code must provide the parameter while users can ignore it.
/// Forgetting to set it in internal code could lead to some bugs which are nasty to find.
pub type ForeignModel<M, T = <<M as Model>::Primary as RawField>::Type> = ForeignModelByField<M, T>;

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
