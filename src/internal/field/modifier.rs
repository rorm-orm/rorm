//! Holds the [`AnnotationsModifier`] trait and some of its implementations

use std::marker::PhantomData;

use crate::fields::traits::FieldType;
use crate::internal::const_concat::ConstString;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::Field;
use crate::internal::hmr::annotations::Annotations;
use crate::internal::hmr::db_type::DbType;
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

/// Trait used in [`FieldType`] to allow types to implement custom compile time checks
///
/// It mimics a `const fn<F: Field>() -> Result<(), &'static str>`,
/// i.e. a `const` function which takes a field `F: Field` as "argument" and produces a `Result<(), &'static str>`,
/// which is not definable using existing `Fn` traits.
///
/// **BEWARE** an implementation is not allowed to access `F::CHECK`!
pub trait CheckModifier<F: Field> {
    /// The check's result
    const RESULT: Result<(), ConstString<1024>>;
}

/// [`CheckModifier`] which checks nothing
pub struct NoCheck;
impl<F: Field> CheckModifier<F> for NoCheck {
    const RESULT: Result<(), ConstString<1024>> = Ok(());
}

/// [`CheckModifier`] which:
/// - requires `F::EFFECTIVE_ANNOTATIONS` to be `Some`
/// - ensures all annotations required by `D` are set
/// - runs the shared linter from `rorm-declaration`
pub struct SingleColumnCheck<D: DbType>(pub PhantomData<D>);

impl<D: DbType, F: Field> CheckModifier<F> for SingleColumnCheck<D> {
    const RESULT: Result<(), ConstString<1024>> = {
        'result: {
            let Some(annotations) = F::EFFECTIVE_ANNOTATIONS else {
                break 'result Err(ConstString::error(&["annotations have been erased"]));
            };

            // Are required annotations set?
            let mut required = D::REQUIRED;
            while let [head, tail @ ..] = required {
                required = tail;
                if !annotations.is_set(head) {
                    break 'result Err(ConstString::error(&[
                        "missing annotation: ",
                        head.as_str(),
                    ]));
                }
            }

            // Run the annotations lint shared with rorm-cli
            let annotations = annotations.as_lint();
            if let Err(err) = annotations.check() {
                break 'result Err(ConstString::error(&["invalid annotations: ", err]));
            }

            Ok(())
        }
    };
}

/// Trait used in [`FieldType`] to derive column names from the field name
///
/// It mimics a `const fn<F: Field>() -> F::Type::Columns<&'static str>`,
/// i.e. a `const` function which takes a field `F: Field` as "argument" and produces a `F::Type::Columns<&'static str>`,
/// which is not definable using existing `Fn` traits.
pub trait ColumnsFromName<F: Field> {
    /// The field's columns' names
    const COLUMNS: <F::Type as FieldType>::Columns<&'static str>;
}

/// [`ColumnsFromName`] for field types which map to no columns
pub struct NoColumnFromName;
impl<F: Field> ColumnsFromName<F> for NoColumnFromName
where
    F::Type: FieldType<Columns<&'static str> = [&'static str; 0]>,
{
    const COLUMNS: <F::Type as FieldType>::Columns<&'static str> = [];
}

/// [`ColumnsFromName`] for field types which map to a single column
pub struct SingleColumnFromName;
impl<F: Field> ColumnsFromName<F> for SingleColumnFromName
where
    F::Type: FieldType<Columns<&'static str> = [&'static str; 1]>,
{
    const COLUMNS: <F::Type as FieldType>::Columns<&'static str> = [F::NAME];
}
