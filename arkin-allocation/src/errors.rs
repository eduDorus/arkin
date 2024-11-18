use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllocationOptimError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
