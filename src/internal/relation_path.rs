//! Implicit join prototypes

use std::marker::PhantomData;

use crate::fields::types::{BackRef, ForeignModelByField};
use crate::internal::const_concat::ConstString;
use crate::internal::field::foreign_model::{ForeignModelField, ForeignModelTrait};
use crate::internal::field::{RawField, SingleColumnField};
use crate::internal::query_context::QueryContext;
use crate::{sealed, Model};

/// Trait to store a relation path in generics
///
/// Paths are constructed nesting [`PathStep`](PathStep)s and terminating the last one with `()`:
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
    sealed!(trait);

    /// The model (or table in the context of joins) this path originates from
    type Origin: Model;

    /// Is `Self = Self::Origin`?
    const IS_ORIGIN: bool = false;

    /// Add all joins required to use this path to the query context
    fn add_to_context(context: &mut QueryContext);
}
impl<M: Model> Path for M {
    sealed!(impl);

    type Origin = M;

    const IS_ORIGIN: bool = true;

    fn add_to_context(_context: &mut QueryContext) {}
}

/// A single step in a [`Path`]
#[derive(Copy, Clone)]
pub struct PathStep<F, P: Path>(PhantomData<(F, P)>);

impl<F, P> Path for PathStep<F, P>
where
    F: RawField + 'static,
    P: Path,
    Self: PathImpl<F::Type>,
{
    sealed!(impl);

    type Origin = P::Origin;

    fn add_to_context(context: &mut QueryContext) {
        <Self as PathImpl<_>>::add_to_context(context);
    }
}
impl<FF, F, P> PathImpl<ForeignModelByField<FF>> for PathStep<F, P>
where
    FF: SingleColumnField,
    F: ForeignModelField<Type = ForeignModelByField<FF>> + 'static,
    P: Path,
{
    sealed!(impl);

    type ResolvedRelatedField = FF;

    const JOIN_FIELDS: [[&'static str; 2]; 2] = [
        [Self::ALIAS, Self::ResolvedRelatedField::NAME],
        [P::ALIAS, F::NAME],
    ];

    fn add_to_context(context: &mut QueryContext) {
        context.add_relation_path::<FF::Model, F, P>();
    }
}
impl<FF, F, P> PathImpl<Option<ForeignModelByField<FF>>> for PathStep<F, P>
where
    FF: SingleColumnField,
    F: ForeignModelField<Type = Option<ForeignModelByField<FF>>> + 'static,
    P: Path,
{
    sealed!(impl);

    type ResolvedRelatedField = FF;

    const JOIN_FIELDS: [[&'static str; 2]; 2] = [
        [Self::ALIAS, Self::ResolvedRelatedField::NAME],
        [P::ALIAS, F::NAME],
    ];

    fn add_to_context(context: &mut QueryContext) {
        context.add_relation_path::<FF::Model, F, P>();
    }
}
impl<FMF, F, P> PathImpl<BackRef<FMF>> for PathStep<F, P>
where
    FMF: ForeignModelField,
    FMF::Type: ForeignModelTrait,
    F: RawField<Type = BackRef<FMF>> + 'static,
    P: Path,
{
    sealed!(impl);

    type ResolvedRelatedField = FMF;

    const JOIN_FIELDS: [[&'static str; 2]; 2] = [
        [Self::ALIAS, Self::ResolvedRelatedField::NAME],
        [
            P::ALIAS,
            <<FMF as RawField>::Type as ForeignModelTrait>::RelatedField::NAME,
        ],
    ];

    fn add_to_context(context: &mut QueryContext) {
        context.add_relation_path::<FMF::Model, F, P>();
    }
}
/// Implementation for [`PathStep`]
///
/// This is a trait instead of a normal `impl` block,
/// because different implementations based on the field's raw type are required.
/// By making this trait generic of this type, these different implementations don't overlap.
/// Also by making this a trait, constants and type aliases can be used as well.
///
/// [`Path`] is implemented generically using [`PathImpl`].
pub trait PathImpl<RawType> {
    sealed!(trait);

    /// The related field the [`PathStep`]'s field points to.
    type ResolvedRelatedField: RawField;

    /// The two field joined on.
    const JOIN_FIELDS: [[&'static str; 2]; 2];

    /// Add all joins required to use this path to the query context
    fn add_to_context(context: &mut QueryContext);
}
/// Shorthand for accessing [`PathImpl::ResolvedRelatedField`](PathImpl::ResolvedRelatedField).
pub type ResolvedRelatedField<F, P> =
    <PathStep<F, P> as PathImpl<<F as RawField>::Type>>::ResolvedRelatedField;

/// Trait shared by [`Path`] and [`FieldProxy`](super::field::FieldProxy) which provides a unique join alias at compile time.s
pub trait JoinAlias {
    sealed!(trait);

    /// Unique join alias
    const ALIAS: &'static str;
}

impl<M: Model> JoinAlias for M {
    sealed!(impl);

    const ALIAS: &'static str = M::TABLE;
}

impl<F: RawField, P: Path> JoinAlias for PathStep<F, P> {
    sealed!(impl);

    const ALIAS: &'static str = ConstString::join_alias(&[P::ALIAS, "__", F::NAME]).as_str();
}
