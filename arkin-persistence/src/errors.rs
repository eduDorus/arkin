use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
