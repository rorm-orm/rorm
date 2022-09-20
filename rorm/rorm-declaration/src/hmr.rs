//! This module holds the high level model representation
//!
//! It adds:
//! - a type level version of [`imr::DbType`] to be used in generic type bound checks
use crate::imr;

/// Trait to associate the type-level db types with their runtime db types
pub trait DbType: 'static {
    /// Equivalent runtime db type
    const IMR: imr::DbType;
}

macro_rules! impl_db_types {
    ($($type:ident,)*) => {
        $(
            pub struct $type;
            impl DbType for $type {
                const IMR: imr::DbType = imr::DbType::$type;
            }
        )*
    };
}

impl_db_types!(
    VarChar, VarBinary, Int8, Int16, Int32, Int64, UInt8, UInt16, UInt32, Float, Double, Boolean,
    Date, Datetime, Timestamp, Time, Choices, Set,
);
