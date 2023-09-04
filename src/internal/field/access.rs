//! Experimental trait to hide a [`FieldProxy`]s two generics behind a single one.

use crate::internal::field::{FieldProxy, RawField};
use crate::internal::relation_path::Path;

macro_rules! FieldType {
    () => {
        <Self::Field as RawField>::Type
    };
}

/// Experimental trait to hide a [`FieldProxy`]s two generics behind a single one.
pub trait FieldAccess: Sized + Send + 'static {
    /// Field which is accessed
    type Field: RawField;

    /// Path the field is accessed through
    type Path: Path;
}

impl<F: RawField, P: Path> FieldAccess for FieldProxy<F, P> {
    type Field = F;
    type Path = P;
}
