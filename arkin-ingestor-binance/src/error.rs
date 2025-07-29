use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestorError {
    #[error("Unknown error")]
    Unknown,
}
