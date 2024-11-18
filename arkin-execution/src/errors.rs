use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderManagerError {
    #[error(transparent)]
    ExecutorError(#[from] ExecutorError),

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Network error occurred: {0}")]
    NetworkError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("API limit exceeded")]
    ApiLimitExceeded,

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Unknown error")]
    Unknown,
}
