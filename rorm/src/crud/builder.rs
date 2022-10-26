//! This module provides primitives used by the various builder.

use rorm_db::conditional::Condition;

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
use private::Private;
use rorm_db::transaction::Transaction;

/// This trait can not be implemented by foreign packages.
///
/// It is the base for marker traits used in our builders.
pub trait Sealed {
    #[doc(hidden)]
    fn sealed<P: Private>() {}
}
impl Sealed for () {}

/// Marker for the generic parameter storing a db [Condition]
pub trait ConditionMarker<'a>: Sealed + 'a {
    /// Get the generic condition as [Option] expected by [rorm_db]
    fn as_option(&self) -> Option<&Condition<'a>>;
}

impl<'a> Sealed for Condition<'a> {}
impl<'a> ConditionMarker<'a> for Condition<'a> {
    fn as_option(&self) -> Option<&Condition<'a>> {
        Some(self)
    }
}
impl<'a> ConditionMarker<'a> for () {
    fn as_option(&self) -> Option<&Condition<'a>> {
        None
    }
}

/// Marker for the generic parameter storing a db [Transaction]
pub trait TransactionMarker<'rf, 'db>: Sealed + 'rf {
    /// Convert the generic transaction into [Option] expected by [rorm_db]
    fn into_option(self) -> Option<&'rf mut Transaction<'db>>;
}

impl<'rf, 'db: 'rf> Sealed for &'rf mut Transaction<'db> {}
impl<'rf, 'db: 'rf> TransactionMarker<'rf, 'db> for &'rf mut Transaction<'db> {
    fn into_option(self) -> Option<&'rf mut Transaction<'db>> {
        Some(self)
    }
}
impl<'rf, 'db> TransactionMarker<'rf, 'db> for () {
    fn into_option(self) -> Option<&'rf mut Transaction<'db>> {
        None
    }
}
