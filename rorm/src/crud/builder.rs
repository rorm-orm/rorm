//! This module provides primitives used by the various builder in [`rorm::crud`]

use rorm_db::conditional::Condition;

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
use private::Private;

/// This trait can not be implemented by foreign packages.
///
/// It is the base for marker traits used in our builders.
pub trait Sealed {
    #[doc(hidden)]
    fn sealed<P: Private>() {}
}

/// Marker for the generic parameter storing a db [Condition]
pub trait ConditionMarker<'a>: Sealed + 'a {
    /// Get the generic condition as [Option] expected by [rorm_db]
    fn as_option(&self) -> Option<&Condition<'a>>;
}

impl Sealed for () {}
impl<'a> ConditionMarker<'a> for () {
    fn as_option(&self) -> Option<&Condition<'a>> {
        None
    }
}

impl<'a> Sealed for Condition<'a> {}
impl<'a> ConditionMarker<'a> for Condition<'a> {
    fn as_option(&self) -> Option<&Condition<'a>> {
        Some(self)
    }
}
