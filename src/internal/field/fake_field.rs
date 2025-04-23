//! Utility for implementing a [`FieldDecoder`](crate::internal::field::decoder::FieldDecoder)
//! on a [`FieldType`] which uses another `FieldType`'s decoder internally.
//!
//! `FieldDecoder::new` takes a field of your type, so when you need to implement your decoder
//! in terms of another one, you need a method of "swapping" the field's type.
//! This is what [`FakeField`] does.

use std::marker::PhantomData;

use crate::fields::traits::{FieldColumns, FieldType};
use crate::fields::utils::column_name::ColumnName;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::Source;

/// Take a field `F` and create a new "fake" field with the different [`Field::Type`] `T`
#[allow(non_camel_case_types)]
pub struct FakeField<T, F>(PhantomData<(T, F)>);
impl<T, F> Clone for FakeField<T, F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, F> Copy for FakeField<T, F> {}
impl<T, F> Field for FakeField<T, F>
where
    T: FieldType<Columns = <F::Type as FieldType>::Columns> + 'static,
    F: Field,
{
    type Type = T;
    type Model = F::Model;
    const INDEX: usize = F::INDEX;
    const NAME: &'static str = F::NAME;
    const EXPLICIT_ANNOTATIONS: Annotations = F::EXPLICIT_ANNOTATIONS;
    const EFFECTIVE_ANNOTATIONS: FieldColumns<F::Type, Annotations> = F::EFFECTIVE_ANNOTATIONS;
    const EFFECTIVE_NAMES: FieldColumns<F::Type, ColumnName> = F::EFFECTIVE_NAMES;
    const SOURCE: Source = F::SOURCE;
    fn new() -> Self {
        Self(PhantomData)
    }
}
