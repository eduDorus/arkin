use std::sync::Arc;

use thiserror::Error;

use arkin_core::prelude::*;

#[derive(Error, Debug)]
pub enum AllocationOptimError {
    #[error("No price data available for instrument {0}")]
    NoPriceDataAvailable(Arc<Instrument>),

    #[error("No execution order send for signal: {0}")]
    NoOrdersToPublish(Arc<Signal>),

    #[error(transparent)]
    PersistenceError(#[from] arkin_persistence::PersistenceError),

    #[error(transparent)]
    AccountingError(#[from] arkin_accounting::AccountingError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
