use std::error::Error;
use std::fmt;

use crate::row::{OwnedRowIndex, RowIndex};

/// Errors returned by [`Row::get`](super::Row::get)
#[derive(Debug)]
pub enum RowError<'i> {
    /// The requested index was not found
    NotFound {
        /// The index which has not been found
        index: RowIndex<'i>,
    },
    /// The type returned by the database and the type expected by rust don't match.
    ///
    /// This is checked before decoding is even attempted.
    MismatchedTypes {
        /// The index the error occurred at
        index: RowIndex<'i>,
        /// The `type_name` of the expected type
        rust_type: &'static str,
    },
    /// An unexpected `NULL` was encountered during decoding.
    ///
    /// This is a special case of `RowError::Decode`
    /// which the caller could handle by wrapping the type to decode in an `Option`.
    UnexpectedNull {
        /// The index the error occurred at
        index: RowIndex<'i>,
    },
    /// An error occurred while decoding the value.
    Decode {
        /// The index the error occurred at
        index: RowIndex<'i>,
        /// The error produced by the `Decode` implementation
        source: Box<dyn Error + Send + Sync>,
    },
    /// An unknown error occurred.
    Unknown {
        /// The index the error occurred at
        index: RowIndex<'i>,
        /// The underlying error which could not be mapped to one of this enum's variants.
        ///
        /// This is an `sqlx::Error` which should not happen according to their documentation.
        source: Box<dyn Error + Send + Sync>,
    },
}

impl<'i> RowError<'i> {
    /// The index the error occurred at
    pub fn index(&self) -> RowIndex<'i> {
        match self {
            RowError::NotFound { index, .. }
            | RowError::MismatchedTypes { index, .. }
            | RowError::UnexpectedNull { index, .. }
            | RowError::Decode { index, .. }
            | RowError::Unknown { index, .. } => *index,
        }
    }

    /// Converts the error into its owned version
    pub fn into_owned(self) -> OwnedRowError {
        match self {
            RowError::NotFound { index } => OwnedRowError::NotFound {
                index: index.into_owned(),
            },
            RowError::MismatchedTypes { index, rust_type } => OwnedRowError::MismatchedTypes {
                index: index.into_owned(),
                rust_type,
            },
            RowError::UnexpectedNull { index } => OwnedRowError::UnexpectedNull {
                index: index.into_owned(),
            },
            RowError::Decode { index, source } => OwnedRowError::Decode {
                index: index.into_owned(),
                source,
            },
            RowError::Unknown { index, source } => OwnedRowError::Unknown {
                index: index.into_owned(),
                source,
            },
        }
    }
}

/// Owned version of [`RowError`]
#[derive(Debug)]
pub enum OwnedRowError {
    /// The requested index was not found
    NotFound {
        /// The index which has not been found
        index: OwnedRowIndex,
    },
    /// The type returned by the database and the type expected by rust don't match.
    ///
    /// This is checked before decoding is even attempted.
    MismatchedTypes {
        /// The index the error occurred at
        index: OwnedRowIndex,
        /// The `type_name` of the expected type
        rust_type: &'static str,
    },
    /// An unexpected `NULL` was encountered during decoding.
    ///
    /// This is a special case of `OwnedRowError::Decode`
    /// which the caller could handle by wrapping the type to decode in an `Option`.
    UnexpectedNull {
        /// The index the error occurred at
        index: OwnedRowIndex,
    },
    /// An error occurred while decoding the value.
    Decode {
        /// The index the error occurred at
        index: OwnedRowIndex,
        /// The error produced by the `Decode` implementation
        source: Box<dyn Error + Send + Sync>,
    },
    /// An unknown error occurred.
    Unknown {
        /// The index the error occurred at
        index: OwnedRowIndex,
        /// The underlying error which could not be mapped to one of this enum's variants.
        ///
        /// This is an `sqlx::Error` which should not happen according to their documentation.
        source: Box<dyn Error + Send + Sync>,
    },
}

impl OwnedRowError {
    /// The index the error occurred at
    pub fn index(&self) -> &OwnedRowIndex {
        match self {
            OwnedRowError::NotFound { index, .. }
            | OwnedRowError::MismatchedTypes { index, .. }
            | OwnedRowError::UnexpectedNull { index, .. }
            | OwnedRowError::Decode { index, .. }
            | OwnedRowError::Unknown { index, .. } => index,
        }
    }
}

impl From<OwnedRowError> for crate::Error {
    fn from(value: OwnedRowError) -> Self {
        crate::Error::RowError(value)
    }
}

impl<'i> From<RowError<'i>> for OwnedRowError {
    fn from(value: RowError<'i>) -> Self {
        value.into_owned()
    }
}

impl<'i> From<RowError<'i>> for crate::Error {
    fn from(value: RowError<'i>) -> Self {
        value.into_owned().into()
    }
}

macro_rules! impl_display {
    (impl$(<$lifetime:lifetime>)? fmt::Display for $Enum:ty { .. }) => {
        impl$(<$lifetime>)? fmt::Display for $Enum {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Self::NotFound { index } => write!(f, "Column {index:?} was not found"),
                    Self::MismatchedTypes { index, rust_type } => write!(f, "Column {index:?} does not match type {rust_type}"),
                    Self::UnexpectedNull { index } => write!(f, "Column {index:?} is `NULL`; try decoding as `Option`"),
                    Self::Decode { index, source } => write!(f, "Couldn't decode column {index:?}: {source}"),
                    Self::Unknown { index, source } => write!(f, "An unknown error occurred at column {index:?}: {source}"),
                }
            }
        }
    };
}
impl_display!(impl<'i> fmt::Display for RowError<'i> {..});
impl_display!(impl fmt::Display for OwnedRowError {..});

macro_rules! impl_error {
    (impl$(<$lifetime:lifetime>)? Error for $Enum:ty { .. }) => {
        impl$(<$lifetime>)? Error for $Enum {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match self {
                    Self::Decode { source, .. } => Some(source.as_ref()),
                    Self::Unknown { source, .. } => Some(source.as_ref()),
                    _ => None,
                }
            }
        }
    };
}
impl_error!(impl<'i> Error for RowError<'i> {..});
impl_error!(impl Error for OwnedRowError {..});
