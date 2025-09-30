use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    ClickhouseError(#[from] clickhouse::error::Error),

    #[error("Entity not found")]
    NotFound,
}
