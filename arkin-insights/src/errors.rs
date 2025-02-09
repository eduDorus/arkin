use thiserror::Error;

#[derive(Error, Debug)]
pub enum InsightsError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
