//! Traits defining types which can be used as fields.

use crate::conditions::Value;
use crate::internal::array_utils::Array;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::modifier::{AnnotationsModifier, CheckModifier};
use crate::internal::field::{FieldKind, RawField};
use crate::internal::imr;

pub mod cmp;

pub use cmp::*;

/// Base trait for types which are allowed as fields in models
pub trait FieldType: 'static {
    /// The kind of field this type declares
    type Kind: FieldKind;

    /// Array with length specific to the field type
    type Columns<T>: Array<Item = T>;

    /// Construct an array of [`Value`] representing `self` in the database via ownership
    fn into_values(self) -> Self::Columns<Value<'static>>;

    /// Construct an array of [`Value`] representing `self` in the database via borrowing
    fn as_values(&self) -> Self::Columns<Value<'_>>;

    /// Construct an array of [`imr::Field`] representing this type
    fn get_imr<F: RawField<Type = Self>>() -> Self::Columns<imr::Field>;

    /// [`FieldDecoder`] to use for fields of this type
    type Decoder: FieldDecoder<Result = Self>;

    /// `const fn<F: RawField>() -> Option<Annotations>`
    /// to allow modifying the a field's annotations which is of this type
    ///
    /// For example can be used to set `nullable` implicitly for `Option<_>`.
    type AnnotationsModifier<F: RawField<Type = Self>>: AnnotationsModifier<F>;

    /// `const fn<F: RawField>() -> Result<(), &'static str>`
    /// to allow custom compile time checks.
    ///
    /// For example can be used to ensure `String` has a `max_lenght`.
    type CheckModifier<F: RawField<Type = Self>>: CheckModifier<F>;
}
