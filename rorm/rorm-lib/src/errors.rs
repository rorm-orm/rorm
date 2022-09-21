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
}
