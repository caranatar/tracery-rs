use thiserror::Error;

/// The `tracery` error type
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error encountered while parsing a rule
    #[error("Error while parsing tracery: {0}")]
    ParseError(String),

    /// A referenced key does not exist
    #[error("Missing key: {0}")]
    MissingKeyError(String),

    /// Error encountered while parsing JSON input
    #[cfg(feature = "tracery_json")]
    #[error("JSON error {0}")]
    JsonError(#[from] serde_json::Error),
}
