/**
Error type to simplify propagating different error types.
 */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error while building sql.
    #[error("sql build error: {0}")]
    SQLBuildError(String)
}
