use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Engine builder error: {0}")]
    BuilderError(String),
}
