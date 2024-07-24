use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Direction not between bounds: {0}")]
    DirectionError(String),

    #[error("Model unknown venue: {0}")]
    UnknownVenueError(String),

    #[error("Model price: {0}")]
    PriceError(String),
}
