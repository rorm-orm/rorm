//! Traits defining types which can be used as fields.

use crate::conditions::Value;
use crate::internal::array_utils::Array;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::FieldKind;

mod cmp;
pub use cmp::*;

/// The type of field allowed on models
pub trait FieldType {
    /// The kind of field this type declares
    type Kind: FieldKind;

    /// Array with length specific to the field type
    type Columns<T>: Array<Item = T>;

    /// Construct an array of [`Value`] representing `self` in the database via ownership
    fn into_values(self) -> Self::Columns<Value<'static>>;

    /// Construct an array of [`Value`] representing `self` in the database via borrowing
    fn as_values(&self) -> Self::Columns<Value<'_>>;

    /// [`FieldDecoder`] to use for fields of this type
    type Decoder: FieldDecoder<Result = Self>;
}
