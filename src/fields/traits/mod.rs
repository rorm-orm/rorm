//! Traits defining types which can be used as fields.

use crate::conditions::Value;
use crate::internal::array_utils::Array;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::modifier::{AnnotationsModifier, CheckModifier};
use crate::internal::field::Field;
use crate::internal::imr;

pub mod cmp;

pub use cmp::*;
use fancy_const::ConstFn;

/// Base trait for types which are allowed as fields in models
pub trait FieldType: 'static {
    /// Array with length specific to the field type
    type Columns<T>: Array<Item = T>;

    /// Construct an array of [`Value`] representing `self` in the database via ownership
    fn into_values(self) -> Self::Columns<Value<'static>>;

    /// Construct an array of [`Value`] representing `self` in the database via borrowing
    fn as_values(&self) -> Self::Columns<Value<'_>>;

    /// Construct an array of [`imr::Field`] representing this type
    fn get_imr<F: Field<Type = Self>>() -> Self::Columns<imr::Field>;

    /// [`FieldDecoder`] to use for fields of this type
    type Decoder: FieldDecoder<Result = Self>;

    /// `const fn<F: Field>() -> Option<Annotations>`
    /// to allow modifying the a field's annotations which is of this type
    ///
    /// For example can be used to set `nullable` implicitly for `Option<_>`.
    type AnnotationsModifier<F: Field<Type = Self>>: AnnotationsModifier<F>;

    /// `const fn<F: Field>() -> Result<(), &'static str>`
    /// to allow custom compile time checks.
    ///
    /// For example can be used to ensure `String` has a `max_length`.
    type CheckModifier<F: Field<Type = Self>>: CheckModifier<F>;

    /// Function producing colum names from the field's name
    ///
    /// `const fn<F: Field>() -> Self::Columns<&'static str>`
    type ColumnsFromName: ConstFn<(&'static str,), Self::Columns<&'static str>>;
}
