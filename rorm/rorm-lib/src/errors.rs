use crate::FFIString;

/**
Representation of all error codes.
 */
#[repr(C)]
#[derive(Debug)]
pub enum Error<'a> {
    /// Everything's fine, nothing to worry about.
    NoError,
    /// Runtime was destroyed or never created and can therefore not be accessed.
    MissingRuntimeError,
    /// An error occurred while getting or accessing the runtime.
    RuntimeError(FFIString<'a>),
    /// An error occurred while trying to convert a FFIString into a &str due to invalid content
    InvalidStringError,
    /// Configuration error
    ConfigurationError(FFIString<'a>),
    /// Database error
    DatabaseError(FFIString<'a>),
    /// There are no rows left in the stream
    NoRowsLeftInStream,
    /// Column could not be converted in the given type
    ColumnDecodeError,
    /// Column was not found in row
    ColumnNotFoundError,
    /// The index in the row was out of bounds
    ColumnIndexOutOfBoundsError,
}
