use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
