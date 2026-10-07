//! Utilities for implementing multi-column fields

use crate::fields::traits::{FieldColumns, FieldType};
use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::const_concat::ConstString;
use crate::internal::hmr::annotations::Annotations;

/// Constructs arrays by concatenating others
pub struct ArrayBuilder<T, const N: usize> {
    array: [T; N],
    index: usize,
}
impl<T, const N: usize> ArrayBuilder<T, N> {
    /// Constructs a new `ArrayBuilder`
    pub const fn new(array: [T; N]) -> Self {
        Self { array, index: 0 }
    }

    /// Extends `self` by another array
    pub fn extend<const M: usize>(&mut self, other: [T; M]) {
        if M > N - self.index {
            panic!("Called ArrayBuild::extend with to many items (Array already contains {} / {N} elements, can't add {M} more)", self.index);
        }
        for item in other {
            self.array[self.index] = item;
            self.index += 1;
        }
    }

    /// Returns the final array
    pub fn finish(self) -> [T; N] {
        if self.index != N {
            panic!("Called ArrayBuild::finish but the array is not finished yet ({} / {N} items are initialized)", self.index);
        }
        self.array
    }

    /// Extends `self` by another array
    pub const fn extend_const<const M: usize>(&mut self, other: [T; M])
    where
        T: Copy,
    {
        if M > N - self.index {
            panic!("Called ArrayBuild::extend_const with to many items");
        }
        let mut other = other.as_slice();
        while let [item, remaining @ ..] = other {
            other = remaining;
            self.array[self.index] = *item;
            self.index += 1;
        }
    }

    /// Returns the final array
    pub const fn finish_const(self) -> [T; N]
    where
        T: Copy,
    {
        if self.index != N {
            panic!("Called ArrayBuild::finish_const but the array is not finished yet");
        }
        self.array
    }
}

/// Marker exposing [`Annotations::empty`] as argument for a [`ConstFn`]
pub struct EmptyAnnotations;
impl Contains<Annotations> for EmptyAnnotations {
    const ITEM: Annotations = Annotations::empty();
}

/// Checks a subfield for correctness by evaluating its [`FieldType`]'s `Check`
///
/// A subfield's [`Field`] type is generic over its parent's [`Field`].
/// However, you can't set explicit annotations on the parent (a multi-column field).
/// This means the check does not depend on the parent.
/// This function is a workaround to call the `FieldType`'s `Check`
/// without using on the normal machinery which would expect a [`Field`].
#[allow(clippy::result_large_err, reason = "There is no heap in const")]
pub const fn check<T: FieldType, A: Contains<Annotations>>() -> Result<(), ConstString<1024>> {
    <<<T as FieldType>::Check as ConstFn<_, _>>::Body<(
        A,
        <<T as FieldType>::GetAnnotations as ConstFn<
            (Annotations,),
            FieldColumns<T, Annotations>,
        >>::Body<(A,)>,
    )> as Contains<_>>::ITEM
}
