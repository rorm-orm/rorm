//! Implicit join prototypes

use std::marker::PhantomData;

use crate::internal::field::Field;
use crate::internal::query_context::QueryContextBuilder;
use crate::{const_concat, sealed, ForeignModel, Model};

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
pub trait Path: JoinAlias + 'static {
    sealed!();

    /// The model (or table in the context of joins) this path originates from
    type Origin: Model;

    /// Add all joins required to use this path to the builder
    fn add_to_join_builder(builder: &mut QueryContextBuilder);
}
impl<M: Model> Path for M {
    type Origin = M;

    fn add_to_join_builder(_builder: &mut QueryContextBuilder) {}
}

/// A single step in a [Path]
#[derive(Copy, Clone)]
pub struct PathStep<F, P: Path>(PhantomData<(F, P)>);

impl<M, F, P> Path for PathStep<F, P>
where
    M: Model,
    F: Field<Type = ForeignModel<M>> + 'static,
    P: Path,
{
    type Origin = P::Origin;

    fn add_to_join_builder(builder: &mut QueryContextBuilder) {
        builder.add_relation_path::<M, F, P>()
    }
}

/// Trait shared by [Path] and [FieldProxy](super::field::FieldProxy) which provides a unique join alias at compile time.s
pub trait JoinAlias {
    sealed!();

    /// Unique join alias
    const ALIAS: &'static str;
}

impl<M: Model> JoinAlias for M {
    const ALIAS: &'static str = M::TABLE;
}

impl<F: Field, P: Path> JoinAlias for PathStep<F, P> {
    const ALIAS: &'static str = const_concat!(&[P::ALIAS, "__", F::NAME]);
}
