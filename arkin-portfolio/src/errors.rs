use std::sync::Arc;

use thiserror::Error;

use arkin_core::prelude::*;

#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Currency mismatch in transfer: {0}")]
    CurrencyMismatch(Arc<Transfer>),

    #[error("Insufficient balance in debit account: {0}")]
    InsufficientBalance(Arc<Transfer>),

    #[error("Transfer amount must be > 0: {0}")]
    InvalidTransferAmount(Arc<Transfer>),

    #[error("Debit account not found: {0}")]
    DebitAccountNotFound(String),

    #[error("Credit account not found: {0}")]
    CreditAccountNotFound(String),

    #[error("Liquidity Account not found for {0}: {1}")]
    LiquidityAccountNotFound(Arc<Venue>, Tradable),

    #[error("Venue Account not found for {0}: {1}")]
    VenueAccountNotFound(Arc<Venue>, Tradable),

    #[error("Strategy Account not found for {0}: {1}")]
    StrategyAccountNotFound(Arc<Strategy>, Tradable),

    #[error("Invalid exchange: {0}")]
    InvalidExchange(String),
}
