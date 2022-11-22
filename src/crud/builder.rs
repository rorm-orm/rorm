//! This module provides primitives used by the various builder.

use rorm_db::conditional as sql;

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
use crate::conditions::Condition;
use crate::internal::query_context::{QueryContext, QueryContextBuilder};
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

/// Marker for the generic parameter storing an optional [Condition]
pub trait ConditionMarker<'a>: Sealed + 'a {
    /// Prepare a query context to be able to handle this condition by registering all implicit joins.
    fn add_to_builder(&self, builder: &mut QueryContextBuilder);

    /// Convert the condition into rorm-sql's format using a query context's registered joins.
    fn into_option<'c>(self, context: &'c QueryContext) -> Option<sql::Condition<'c>>
    where
        'a: 'c;
}

impl<'a> ConditionMarker<'a> for () {
    fn add_to_builder(&self, _builder: &mut QueryContextBuilder) {}

    fn into_option<'c>(self, _context: &'c QueryContext) -> Option<sql::Condition<'c>>
    where
        'a: 'c,
    {
        None
    }
}

impl<'a, T: Condition<'a>> Sealed for T {}
impl<'a, T: Condition<'a>> ConditionMarker<'a> for T {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        Condition::add_to_builder(self, builder);
    }

    fn into_option<'c>(self, context: &'c QueryContext) -> Option<sql::Condition<'c>>
    where
        'a: 'c,
        Self: 'c,
    {
        Some(self.as_sql(context))
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
