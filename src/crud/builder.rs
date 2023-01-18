//! This module provides primitives used by the various builder.

use crate::conditions::Condition;
use crate::internal::query_context::QueryContextBuilder;
use crate::sealed;

/// Marker for the generic parameter storing an optional [`Condition`]
pub trait ConditionMarker<'a>: 'a {
    sealed!();

    /// Prepare a query context to be able to handle this condition by registering all implicit joins.
    fn add_to_builder(&self, builder: &mut QueryContextBuilder);

    /// Convert the condition into rorm-sql's format using a query context's registered joins.
    fn into_option(self) -> Option<Box<dyn Condition<'a>>>;
}

impl<'a> ConditionMarker<'a> for () {
    fn add_to_builder(&self, _builder: &mut QueryContextBuilder) {}

    fn into_option(self) -> Option<Box<dyn Condition<'a>>> {
        None
    }
}

impl<'a, T: Condition<'a>> ConditionMarker<'a> for T {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        Condition::add_to_builder(self, builder);
    }

    fn into_option(self) -> Option<Box<dyn Condition<'a>>> {
        Some(self.boxed())
    }
}
