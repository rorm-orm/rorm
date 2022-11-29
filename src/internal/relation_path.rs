//! Implicit join prototypes

use std::marker::PhantomData;

use crate::internal::field::back_ref::BackRef;
use crate::internal::field::foreign_model::ForeignModelByField;
use crate::internal::field::{foreign_model, RawField};
use crate::internal::query_context::QueryContextBuilder;
use crate::{const_concat, sealed, Model};

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

impl<F, P> Path for PathStep<F, P>
where
    F: RawField + 'static,
    P: Path,
    Self: PathImpl<F::RawType>,
{
    type Origin = P::Origin;

    fn add_to_join_builder(builder: &mut QueryContextBuilder) {
        <Self as PathImpl<_>>::add_to_join_builder(builder);
    }
}
impl<T, M, F, P> PathImpl<ForeignModelByField<M, T>> for PathStep<F, P>
where
    M: Model,
    F: RawField<RawType = ForeignModelByField<M, T>> + 'static,
    P: Path,
{
    type ResolvedRelatedField = foreign_model::RelatedField<M, F>;

    const JOIN_FIELDS: [[&'static str; 2]; 2] = [
        [Self::ALIAS, Self::ResolvedRelatedField::NAME],
        [P::ALIAS, F::NAME],
    ];

    fn add_to_join_builder(builder: &mut QueryContextBuilder) {
        builder.add_relation_path::<M, F, P>();
    }
}
impl<T, M, F, RF, P> PathImpl<BackRef<M>> for PathStep<F, P>
where
    M: Model,
    F: RawField<RawType = BackRef<M>, RelatedField = RF> + 'static,
    RF: RawField<RawType = ForeignModelByField<F::Model, T>>,
    P: Path,
{
    type ResolvedRelatedField = RF;

    const JOIN_FIELDS: [[&'static str; 2]; 2] = [
        [Self::ALIAS, Self::ResolvedRelatedField::NAME],
        [P::ALIAS, foreign_model::RelatedField::<F::Model, RF>::NAME],
    ];

    fn add_to_join_builder(builder: &mut QueryContextBuilder) {
        builder.add_relation_path::<M, F, P>();
    }
}
/// Implementation for [PathStep]
///
/// This is a trait instead of a normal `impl` block,
/// because different implementations based on the field's raw type are required.
/// By making this trait generic of this type, these different implementations don't overlap.
/// Also by making this a trait, constants and type aliases can be used as well.
///
/// [Path] is implemented generically using [PathImpl].
pub trait PathImpl<RawType> {
    /// The related field the [PathStep]'s field points to.
    ///
    /// This type ensures the [RawField]'s [RelatedField](RawField::RelatedField) is unpacked properly.
    type ResolvedRelatedField: RawField;

    /// The two field joined on.
    const JOIN_FIELDS: [[&'static str; 2]; 2];

    /// Add all joins required to use this path to the builder
    fn add_to_join_builder(builder: &mut QueryContextBuilder);
}
/// Shorthand for accessing [PathImpl::ResolvedRelatedField](PathImpl::ResolvedRelatedField).
pub type ResolvedRelatedField<F, P> =
    <PathStep<F, P> as PathImpl<<F as RawField>::RawType>>::ResolvedRelatedField;

/// Trait shared by [Path] and [FieldProxy](super::field::FieldProxy) which provides a unique join alias at compile time.s
pub trait JoinAlias {
    sealed!();

    /// Unique join alias
    const ALIAS: &'static str;
}

impl<M: Model> JoinAlias for M {
    const ALIAS: &'static str = M::TABLE;
}

impl<F: RawField, P: Path> JoinAlias for PathStep<F, P> {
    const ALIAS: &'static str = const_concat!(&[P::ALIAS, "__", F::NAME]);
}
