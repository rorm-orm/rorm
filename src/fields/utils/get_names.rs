//! Re-usable implementations of [`FieldType::GetNames`]

use generic_array::arr;
use generic_array::typenum::{U0, U1};
use generic_array::GenericArray;

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;
use crate::fields::utils::column_name::ColumnName;

const_fn! {
    /// [`FieldType::GetNames`] for fields without columns
    pub fn no_columns_names(_field_name: ColumnName) -> GenericArray<ColumnName, U0> {
        arr![]
    }
}

const_fn! {
    /// [`FieldType::GetNames`] for fields with a single column
    pub fn single_column_name(field_name: ColumnName) -> GenericArray<ColumnName, U1> {
        arr![field_name]
    }
}
