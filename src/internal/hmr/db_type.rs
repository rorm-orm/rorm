//! A type level version of [`imr::DbType`](crate::imr::DbType) to be used in generic type bound checks

use super::AsImr;
use crate::internal::hmr::annotations::AnnotationIndex;
use crate::{declare_type_option, imr, sealed};

/// Trait to associate the type-level db types with their runtime db types
pub trait DbType: 'static {
    sealed!();

    /// Equivalent runtime db type
    const IMR: imr::DbType;

    /// Annotations required by this type
    const REQUIRED: &'static [AnnotationIndex] = &[];
}

impl<T: DbType> AsImr for T {
    type Imr = imr::DbType;

    fn as_imr(&self) -> Self::Imr {
        T::IMR
    }
}

macro_rules! impl_db_types {
        ($(#[doc = $doc:literal] $type:ident $(requires $required:expr)?,)*) => {
            $(
                #[doc = $doc]
                pub struct $type;
                impl DbType for $type {
                    const IMR: imr::DbType = imr::DbType::$type;

                    $(const REQUIRED: &'static [AnnotationIndex] = &$required;)?
                }
            )*
        };
    }

impl_db_types!(
    /// Type level version of [`imr::DbType::VarChar`]
    VarChar requires [AnnotationIndex::MaxLength],
    /// Type level version of [`imr::DbType::VarBinary`]
    VarBinary,
    /// Type level version of [`imr::DbType::Int8`]
    Int8,
    /// Type level version of [`imr::DbType::Int16`]
    Int16,
    /// Type level version of [`imr::DbType::Int32`]
    Int32,
    /// Type level version of [`imr::DbType::Int64`]
    Int64,
    /// Type level version of [`imr::DbType::Float`]
    Float,
    /// Type level version of [`imr::DbType::Double`]
    Double,
    /// Type level version of [`imr::DbType::Boolean`]
    Boolean,
    /// Type level version of [`imr::DbType::Date`]
    Date,
    /// Type level version of [`imr::DbType::DateTime`]
    DateTime,
    /// Type level version of [`imr::DbType::Timestamp`]
    Timestamp,
    /// Type level version of [`imr::DbType::Time`]
    Time,
    /// Type level version of [`imr::DbType::Choices`]
    Choices,
);

declare_type_option!(OptionDbType, DbType);
