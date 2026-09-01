/// Error type to simplify propagating different error types.
use std::{error, fmt};

/// Error type to simplify propagating different error types.
#[derive(Debug)]
pub enum Error {
    /// Error while building sql.
    SQLBuildError(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SQLBuildError(error) => {
                write!(f, "sql build error: {error}")
            }
        }
    }
}
