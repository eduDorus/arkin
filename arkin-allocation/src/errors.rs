use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllocationOptimError {
    #[error(transparent)]
    PersistenceError(#[from] arkin_persistence::PersistenceError),

    #[error(transparent)]
    AccountingError(#[from] arkin_accounting::AccountingError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
