//! Re-usable implementations of [`FieldType::GetAnnotations`]

use generic_array::typenum::U1;
use generic_array::{arr, ArrayLength, GenericArray};

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;
use crate::fields::utils::const_fn::Contains;
use crate::internal::const_concat::ConstString;
use crate::internal::hmr::annotations::Annotations;

const_fn! {
    /// [`FieldType::GetAnnotations`] which merges the field's explicit annotations
    /// with a set of implicit ones provided by `Implicit`.
    pub fn merge_annotations<Implicit: Contains<Annotations>>(field: Annotations) -> GenericArray<Annotations, U1> {
        match field.merge(Implicit::ITEM) {
            Ok(annotations) => arr![annotations],
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
    pub fn forward_annotations<N: ArrayLength>(field: Annotations) -> GenericArray<Annotations, N> {
        let mut array = GenericArray::uninit();
        let mut i = 0;
        while i < array.as_mut_slice().len() {
            array.as_mut_slice()[i].write(field);
            i += 1;
        }
        unsafe {
            // SAFETY: we iterated over the entire array and wrote to every index
            GenericArray::assume_init(array)
        }
    }
}

const_fn! {
    /// [`FieldType::GetAnnotations`] which adds `nullable` to the explicit annotations.
    pub fn set_null_annotations(field: Annotations) -> GenericArray<Annotations, U1> {
        let mut field = field;
        field.nullable = true;
        arr![field]
    }
}
