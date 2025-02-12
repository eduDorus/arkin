use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderManagerError {
    #[error("Instrument already has order: {0}")]
    OrderAlreadyExists(String),

    #[error("ExecutionOrder not found: {0}")]
    ExecutionOrderNotFound(String),

    #[error("VeneueOrder not found: {0}")]
    VenueOrderNotFound(String),

    #[error("Unknown error")]
    Unknown,
}
