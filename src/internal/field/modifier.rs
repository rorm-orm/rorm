//! Holds the [`AnnotationsModifier`] trait and some of its implementations

use std::marker::PhantomData;

use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::RawField;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
use crate::{const_panic, Model};

/// Trait used in [`FieldType`] to allow types to modify their fields' annotations.
///
/// It mimics a `const fn<F: RawField>() -> Option<Annotations>`,
/// i.e. a `const` function which takes a field `F: RawField` as "argument" and produces a `Option<Annotations>`,
/// which is not definable using existing `Fn` traits.
///
/// **BEWARE** an implementation is not allowed to access `F::EFFECTIVE_ANNOTATIONS` or `F::CHECK`!
pub trait AnnotationsModifier<F: RawField> {
    /// The resulting modified annotations
    const MODIFIED: Option<Annotations>;
}

/// [`AnnotationsModifier`] which doesn't modify annotations
pub struct UnchangedAnnotations;
impl<F: RawField> AnnotationsModifier<F> for UnchangedAnnotations {
    const MODIFIED: Option<Annotations> = Some(F::EXPLICIT_ANNOTATIONS);
}

/// [`AnnotationsModifier`] which sets annotations to `None`
pub struct EraseAnnotations;
impl<F: RawField> AnnotationsModifier<F> for EraseAnnotations {
    const MODIFIED: Option<Annotations> = None;
}

/// [`AnnotationsModifier`] which merges the annotations with [`AsDbType`]'s implicit annotations
pub struct MergeAnnotations<T: AsDbType>(pub PhantomData<T>);
impl<T: AsDbType, F: RawField> AnnotationsModifier<F> for MergeAnnotations<T> {
    const MODIFIED: Option<Annotations> = {
        if let Some(implicit) = T::IMPLICIT {
            match F::EXPLICIT_ANNOTATIONS.merge(implicit) {
                Ok(annotations) => Some(annotations),
                Err(duplicate) => {
                    const_panic!(&[
                        "The annotation ",
                        duplicate,
                        " on ",
                        <F::Model as Model>::TABLE,
                        ".",
                        F::NAME,
                        " is implied by its type and can't be set explicitly",
                    ]);
                }
            }
        } else {
            Some(F::EXPLICIT_ANNOTATIONS)
        }
    };
}

/// Trait used in [`FieldType`] to allow types to implement custom compile time checks
///
/// It mimics a `const fn<F: RawField>() -> Result<(), &'static str>`,
/// i.e. a `const` function which takes a field `F: RawField` as "argument" and produces a `Result<(), &'static str>`,
/// which is not definable using existing `Fn` traits.
///
/// **BEWARE** an implementation is not allowed to access `F::CHECK`!
pub trait CheckModifier<F: RawField> {
    /// The check's result
    const RESULT: Result<(), &'static str>;
}

/// [`CheckModifier`] which checks nothing
pub struct NoCheck;
impl<F: RawField> CheckModifier<F> for NoCheck {
    const RESULT: Result<(), &'static str> = Ok(());
}

/// [`CheckModifier`] which:
/// - requires `F::EFFECTIVE_ANNOTATIONS` to be `Some`
/// - ensures all annotations required by `D` are set
/// - runs the shared linter from `rorm-declaration`
pub struct SingleColumnCheck<D: DbType>(pub PhantomData<D>);

impl<D: DbType, F: RawField> CheckModifier<F> for SingleColumnCheck<D> {
    const RESULT: Result<(), &'static str> = {
        'result: {
            let Some(annotations) = F::EFFECTIVE_ANNOTATIONS else {
                break 'result Err("annotations have been erased");
            };

            // Are required annotations set?
            let mut required = D::REQUIRED;
            while let [head, tail @ ..] = required {
                required = tail;
                if !annotations.is_set(head) {
                    // break 'result Err(const_concat!(&["missing annotation: ", head.as_str(),]));
                    break 'result Err(head.as_str());
                }
            }

            // Run the annotations lint shared with rorm-cli
            let annotations = annotations.as_lint();
            if let Err(err) = annotations.check() {
                // break 'result Err(const_concat!(&["invalid annotations: ", err]));
                break 'result Err(err);
            }

            Ok(())
        }
    };
}
