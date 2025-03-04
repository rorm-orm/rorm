//! Re-usable implementations of [`FieldType::GetNames`]

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;

const_fn! {
    /// [`FieldType::GetNames`] for fields without columns
    pub fn no_columns_names(_field_name: &'static str) -> [&'static str; 0] {
        []
    }
}

const_fn! {
    /// [`FieldType::GetNames`] for fields with a single column
    pub fn single_column_name(field_name: &'static str) -> [&'static str; 1] {
        [field_name]
    }
}
