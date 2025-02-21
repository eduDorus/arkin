use std::sync::Arc;

use rust_decimal::Decimal;
use thiserror::Error;

use arkin_core::prelude::*;

#[derive(Error, Debug)]
pub enum AccountingError {
    #[error("Missing strategy information on strategy account creation")]
    MissingStrategy,

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Currency mismatch in transfer: {0}")]
    CurrencyMismatch(Arc<Transfer>),

    #[error("Insufficient balance in debit account: {0}")]
    InsufficientBalance(Arc<Transfer>),

    #[error("Invalid balance: {0}")]
    InvalidBalance(Decimal),

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

    #[error("Invalid instrument: {0}")]
    UnsupportedInstrumentType(InstrumentType),

    #[error("Same account found for transaction: {0}")]
    SameAccount(Arc<Transfer>),
}
