//! Error type to simplify propagating different error types.

use std::{error, fmt};

#[cfg(not(feature = "sqlx"))]
mod sqlx {
    /// Represent all ways a method can fail within SQLx.
    ///
    /// I.e. not at all since sqlx isn't a dependency.
    #[derive(Debug)]
    pub enum Error {}

    impl std::error::Error for Error {}

    impl std::fmt::Display for Error {
        fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
            unreachable!("You shouldn't be able to get an instance of an empty enum!");
        }
    }
}

use sqlx::Error as SqlxError;

use crate::row::OwnedRowError;

/// Error type to simplify propagating different error types.
#[derive(Debug)]
pub enum Error {
    /// Error returned from Sqlx
    SqlxError(SqlxError),

    /// Error for pointing to configuration errors.
    ConfigurationError(String),

    /// SQL building error
    SQLBuildError(rorm_sql::error::Error),

    /// Error returned by [`Row::get`](crate::Row::get)
    RowError(OwnedRowError),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::SqlxError(source) => Some(source),
            Error::ConfigurationError(_) => None,
            Error::SQLBuildError(source) => Some(source),
            Error::RowError(source) => Some(source),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SqlxError(error) => write!(f, "sqlx error: {error}"),
            Error::ConfigurationError(error) => write!(f, "configuration error: {error}",),
            Error::SQLBuildError(error) => {
                write!(f, "sql error: {error}")
            }
            Error::RowError(error) => {
                write!(f, "{error}")
            }
        }
    }
}

impl From<SqlxError> for Error {
    fn from(source: SqlxError) -> Self {
        Error::SqlxError(source)
    }
}

impl From<rorm_sql::error::Error> for Error {
    fn from(source: rorm_sql::error::Error) -> Self {
        Error::SQLBuildError(source)
    }
}
