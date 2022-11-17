//! Implicit join prototypes

use std::marker::PhantomData;

use crate::internal::field::Field;
use crate::internal::query_context::QueryContextBuilder;
use crate::{ForeignModel, Model};

/// Trait to store a relation path in generics
///
/// Paths are constructed nesting [PathSteps](PathStep) and terminating the last one with `()`:
/// ```skip
/// PathStep<A, PathStep<B, PathStep<C, ()>>>
/// ```
///
/// They represent the "path" a field is access through:
/// ```skip
/// // Direct access
/// let _: FieldProxy<__Name, ())>
///     = Group::F.name;
///
/// // Access through a single relation
/// let _: FieldProxy<__Name, PathStep<__Group, ()>>
///     = User::F.group.fields().name;
///
/// // Access through two relation steps
/// let _: FieldProxy<__Name, PathStep<__Group, PathStep<__User, ()>>>
///     = Comment::F.user.fields().group.fields().name;
/// ```
pub trait Path: 'static {
    /// Add all joins required to use this path to the builder
    fn add_to_join_builder(builder: &mut QueryContextBuilder);
}
impl Path for () {
    fn add_to_join_builder(_builder: &mut QueryContextBuilder) {}
}

/// A single step in a [Path]
#[derive(Copy, Clone)]
pub struct PathStep<F, P>(PhantomData<(F, P)>);

impl<M, F, P> PathStep<F, P>
where
    M: Model,
    F: Field<Type = ForeignModel<M>>,
    P: Path,
{
    /// Create a new instance
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
impl<M, F, P> Path for PathStep<F, P>
where
    M: Model,
    F: Field<Type = ForeignModel<M>> + 'static,
    P: Path,
{
    fn add_to_join_builder(builder: &mut QueryContextBuilder) {
        builder.add_relation_path::<M, F, P>()
    }
}
