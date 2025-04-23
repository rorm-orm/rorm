//! Re-usable implementations of [`FieldType::GetNames`]

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;
use crate::fields::utils::column_name::ColumnName;

const_fn! {
    /// [`FieldType::GetNames`] for fields without columns
    pub fn no_columns_names(_field_name: ColumnName) -> [ColumnName; 0] {
        []
    }
}

const_fn! {
    /// [`FieldType::GetNames`] for fields with a single column
    pub fn single_column_name(field_name: ColumnName) -> [ColumnName; 1] {
        [field_name]
    }
}
