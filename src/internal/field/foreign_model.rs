//! The [ForeignModel] field type

use crate::conditions::Value;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{Field, RawField};
use crate::internal::hmr::annotations::Annotations;
use crate::Model;

/// Stores a link to another model in a field.
///
/// In database language, this is a many to one relation.
pub enum ForeignModel<M: Model> {
    /// The other model's primary key which can be used to query it later.
    Key(<M::Primary as Field>::Type),
    /// The other model's queried instance.
    Instance(Box<M>),
}
impl<M: Model> Clone for ForeignModel<M>
where
    M: Clone,
    <M::Primary as Field>::Type: Clone,
{
    fn clone(&self) -> Self {
        match self {
            ForeignModel::Key(primary) => ForeignModel::Key(primary.clone()),
            ForeignModel::Instance(model) => ForeignModel::Instance(model.clone()),
        }
    }
}

impl<M: Model> AsDbType for ForeignModel<M> {
    type Primitive = <<M::Primary as Field>::Type as AsDbType>::Primitive;
    type DbType = <<M::Primary as Field>::Type as AsDbType>::DbType;

    const IMPLICIT: Option<Annotations> = <<M::Primary as Field>::Type as AsDbType>::IMPLICIT;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Self::Key(<M::Primary as Field>::Type::from_primitive(primitive))
    }

    fn as_primitive(&self) -> Value {
        match self {
            ForeignModel::Key(value) => value.as_primitive(),
            ForeignModel::Instance(model) => {
                if let Some(value) = model.get(<M::Primary as RawField>::INDEX) {
                    value
                } else {
                    unreachable!("A model should contain its primary key");
                }
            }
        }
    }

    const IS_NULLABLE: bool = <<M::Primary as Field>::Type as AsDbType>::IS_NULLABLE;

    const IS_FOREIGN: Option<(&'static str, &'static str)> =
        Some((M::TABLE, <M::Primary as RawField>::NAME));
}
