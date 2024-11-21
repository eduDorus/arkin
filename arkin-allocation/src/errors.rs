use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllocationOptimError {
    #[error(transparent)]
    PersistenceError(#[from] arkin_persistence::PersistenceError),

    #[error(transparent)]
    PortfolioError(#[from] arkin_portfolio::PortfolioError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
