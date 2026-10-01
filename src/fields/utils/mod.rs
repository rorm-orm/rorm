//! Utility types, traits and functions required to declare and implement the [`FieldType`](crate::fields::traits::FieldType) trait.

use generic_array::{ArrayLength, GenericArray};

pub mod check;
pub mod column_name;
pub mod const_fn;
pub mod get_annotations;
pub mod get_names;

/// Constructs a new `GenericArray` filled with `value`
pub const fn new_generic_array<T, N>(value: T) -> GenericArray<T, N>
where
    T: Copy,
    N: ArrayLength,
{
    let mut array = GenericArray::uninit();
    let mut i = 0;
    while i < array.as_mut_slice().len() {
        array.as_mut_slice()[i].write(value);
        i += 1;
    }
    unsafe {
        // SAFETY: we iterated over the entire array and wrote to every index
        GenericArray::assume_init(array)
    }
}
