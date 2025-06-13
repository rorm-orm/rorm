//! Utility stuff around [Patch]

use crate::internal::ConstRef;
use crate::model::Patch;
use crate::Model;

/// Helper used by macros to access a [`Model`]'s `Fields` with less syntactic noise
pub const fn model_fields<M: Model>() -> &'static M::Fields<M> {
    ConstRef::REF
}

/// Like [`std::borrow::Cow`] but for internal use
pub enum PatchCow<'p, P: Patch> {
    /// A borrowed patch
    Borrowed(&'p P),

    /// An owned patch
    Owned(P),
}

/// Trait to make APIs generic over owned and borrowed patches
///
/// Will be implemented by [`#derive(Model)`] or [`#derive(Patch)`] for the struct and its reference.
pub trait IntoPatchCow<'a>: 'a {
    /// Either `Self` or the struct `Self` is a reference of
    type Patch: Patch;

    /// Wrap self as [`PatchCow`]
    fn into_patch_cow(self) -> PatchCow<'a, Self::Patch>;
}
