use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Analitics(#[from] crate::analytics::errors::AnalyticsError),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
