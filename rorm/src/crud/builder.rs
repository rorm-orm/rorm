//! This module provides primitives used by the various builder.

use rorm_db::conditional as sql;

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
use crate::conditions::Condition;
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
    fn into_option(self) -> Option<sql::Condition<'a>>;
}

impl<'a> ConditionMarker<'a> for () {
    fn into_option(self) -> Option<sql::Condition<'a>> {
        None
    }
}

impl<'a, T: Condition<'a>> Sealed for T {}
impl<'a, T: Condition<'a>> ConditionMarker<'a> for T {
    fn into_option(self) -> Option<sql::Condition<'a>> {
        Some(self.as_sql())
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
