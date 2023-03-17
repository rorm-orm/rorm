//! This module provides primitives used by the various builder.

use crate::conditions::Condition;
use crate::internal::query_context::QueryContext;
use crate::sealed;

/// Marker for the generic parameter storing an optional [`Condition`]
pub trait ConditionMarker<'a>: 'a + Send {
    sealed!();

    /// Prepare a query context to be able to handle this condition by registering all implicit joins.
    fn add_to_builder(&self, context: &mut QueryContext);

    /// Convert the condition into rorm-sql's format using a query context's registered joins.
    fn into_option(self) -> Option<Box<dyn Condition<'a>>>;
}

impl<'a> ConditionMarker<'a> for () {
    fn add_to_builder(&self, _context: &mut QueryContext) {}

    fn into_option(self) -> Option<Box<dyn Condition<'a>>> {
        None
    }
}

impl<'a, T: Condition<'a>> ConditionMarker<'a> for T {
    fn add_to_builder(&self, context: &mut QueryContext) {
        Condition::add_to_context(self, context);
    }

    fn into_option(self) -> Option<Box<dyn Condition<'a>>> {
        Some(self.boxed())
    }
}
