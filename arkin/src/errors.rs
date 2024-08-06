use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unknown error: {0}")]
    Unknown(String),
}
