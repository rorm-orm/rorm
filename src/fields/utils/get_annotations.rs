//! Re-usable implementations of [`FieldType::GetAnnotations`]

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;
use crate::fields::utils::const_fn::Contains;
use crate::internal::const_concat::ConstString;
use crate::internal::hmr::annotations::Annotations;

const_fn! {
    /// [`FieldType::GetAnnotations`] which merges the field's explicit annotations
    /// with a set of implicit ones provided by `Implicit`.
    pub fn merge_annotations<Implicit: Contains<Annotations>>(field: Annotations) -> [Annotations; 1] {
        match field.merge(Implicit::ITEM) {
            Ok(annotations) => [annotations],
            Err(duplicate) => {
                let error = ConstString::error(&[
                    "The annotation ",
                    duplicate,
                    " is implied by its field's type and can't be set explicitly",
                ]);
                panic!("{}", error.as_str());
            }
        }
    }
}

const_fn! {
    /// [`FieldType::GetAnnotations`] which forwards the field's explicit annotations to every column.
    pub fn forward_annotations<const N: usize>(field: Annotations) -> [Annotations; N] {
        [field; N]
    }
}

const_fn! {
    /// [`FieldType::GetAnnotations`] which adds `nullable` to the explicit annotations.
    pub fn set_null_annotations(field: Annotations) -> [Annotations; 1] {
        let mut field = field;
        field.nullable = true;
        [field]
    }
}
