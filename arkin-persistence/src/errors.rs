use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error("Entity not found")]
    NotFound,
}
