//! A wrapper around integer implying primary_key

use crate::AsDbType;
use rorm_common::imr::{Annotation, DbType};
use std::ops::{Deref, DerefMut};

/// The type to add to most models as primary key:
/// ```ignore
/// use rorm::{Model, id::Id};
///
/// #[derive(Model)]
/// struct SomeModel {
///     id: Id,
///     ..
/// }
pub type ID = GenericId<u64>;

/// Generic Wrapper which implies the primary key and autoincrement annotation
#[derive(Copy, Clone)]
pub struct GenericId<I: AsDbType>(pub I);

impl<I: AsDbType> AsDbType for GenericId<I> {
    fn as_db_type(annotations: &[Annotation]) -> DbType {
        I::as_db_type(annotations)
    }

    fn implicit_annotations() -> Vec<Annotation> {
        let mut annotations = I::implicit_annotations();
        annotations.push(Annotation::PrimaryKey); // TODO check if already
        annotations.push(Annotation::AutoIncrement);
        annotations
    }
}

impl<I: AsDbType> From<I> for GenericId<I> {
    fn from(id: I) -> Self {
        GenericId(id)
    }
}

impl<I: AsDbType> Deref for GenericId<I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<I: AsDbType> DerefMut for GenericId<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
