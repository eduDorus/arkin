use thiserror::Error;

use crate::ledger::Transfer;

#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Currency mismatch in transfer: {0}")]
    CurrencyMismatch(Transfer),

    #[error("Insufficient balance in debit account: {0}")]
    InsufficientBalance(Transfer),

    #[error("Transfer amount must be > 0: {0}")]
    InvalidTransferAmount(Transfer),

    #[error("Debit account not found: {0}")]
    DebitAccountNotFound(String),

    #[error("Credit account not found: {0}")]
    CreditAccountNotFound(String),
}
