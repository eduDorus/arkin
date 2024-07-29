use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("State builder: {0}")]
    BuilderError(String),
}
