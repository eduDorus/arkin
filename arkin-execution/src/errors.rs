use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderManagerError {
    #[error("Instrument already has order: {0}")]
    OrderAlreadyExists(String),

    #[error(transparent)]
    ExecutorError(#[from] ExecutorError),

    #[error("ExecutionOrder not found: {0}")]
    ExecutionOrderNotFound(String),

    #[error("VeneueOrder not found: {0}")]
    VenueOrderNotFound(String),

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("OrderManager error occurred: {0}")]
    OrderManagerError(String),

    #[error("Executor error occurred: {0}")]
    ExecutorError(String),

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

    #[error("Unknown error: {0}")]
    Unknown(String),
}
