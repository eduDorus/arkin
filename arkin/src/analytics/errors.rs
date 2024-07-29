use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Analytica calculation error: {0}")]
    CalculationError(String),
}
