//! Holds the [`AnnotationsModifier`] trait and some of its implementations

use std::marker::PhantomData;

use crate::internal::const_concat::ConstString;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::Model;

/// Trait used in [`FieldType`] to allow types to modify their fields' annotations.
///
/// It mimics a `const fn<F: Field>() -> Option<Annotations>`,
/// i.e. a `const` function which takes a field `F: Field` as "argument" and produces a `Option<Annotations>`,
/// which is not definable using existing `Fn` traits.
///
/// **BEWARE** an implementation is not allowed to access `F::EFFECTIVE_ANNOTATIONS` or `F::CHECK`!
pub trait AnnotationsModifier<F: Field> {
    /// The resulting modified annotations
    const MODIFIED: Option<Annotations>;
}

/// [`AnnotationsModifier`] which doesn't modify annotations
pub struct UnchangedAnnotations;
impl<F: Field> AnnotationsModifier<F> for UnchangedAnnotations {
    const MODIFIED: Option<Annotations> = Some(F::EXPLICIT_ANNOTATIONS);
}

/// [`AnnotationsModifier`] which sets annotations to `None`
pub struct EraseAnnotations;
impl<F: Field> AnnotationsModifier<F> for EraseAnnotations {
    const MODIFIED: Option<Annotations> = None;
}

/// [`AnnotationsModifier`] which merges the annotations with [`AsDbType`]'s implicit annotations
pub struct MergeAnnotations<T: AsDbType>(pub PhantomData<T>);
impl<T: AsDbType, F: Field> AnnotationsModifier<F> for MergeAnnotations<T> {
    const MODIFIED: Option<Annotations> = {
        if let Some(implicit) = T::IMPLICIT {
            match F::EXPLICIT_ANNOTATIONS.merge(implicit) {
                Ok(annotations) => Some(annotations),
                Err(duplicate) => {
                    let error = ConstString::error(&[
                        "The annotation ",
                        duplicate,
                        " on ",
                        <F::Model as Model>::TABLE,
                        ".",
                        F::NAME,
                        " is implied by its type and can't be set explicitly",
                    ]);
                    panic!("{}", error.as_str());
                }
            }
        } else {
            Some(F::EXPLICIT_ANNOTATIONS)
        }
    };
}
