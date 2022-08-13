#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("configuration error: {0}")]
    ConfigurationError(String),
}
