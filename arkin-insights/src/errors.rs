use thiserror::Error;

#[derive(Error, Debug)]
pub enum InsightsError {
    #[error(transparent)]
    Persistence(#[from] arkin_persistence::PersistenceError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
