//! This module provides primitives used by the various builder.

use crate::conditions::Condition;
use crate::internal::query_context::QueryContext;
use crate::sealed;

/// Marker for the generic parameter storing an optional [`Condition`]
pub trait ConditionMarker<'a>: Send {
    sealed!(trait);

    /// Calls [`Condition::build`] if `Self: Condition`
    /// or returns `None` if `Self = ()`
    fn build(&self, ctx: &mut QueryContext) -> Option<rorm_db::sql::conditional::Condition<'a>>;
}

impl<'a> ConditionMarker<'a> for () {
    sealed!(impl);

    fn build(&self, _ctx: &mut QueryContext) -> Option<rorm_db::sql::conditional::Condition<'a>> {
        None
    }
}

impl<'a, T: Condition<'a>> ConditionMarker<'a> for T {
    sealed!(impl);

    fn build(&self, ctx: &mut QueryContext) -> Option<rorm_db::sql::conditional::Condition<'a>> {
        Some(<T as Condition<'a>>::build(self, ctx))
    }
}
