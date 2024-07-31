use async_trait::async_trait;
use std::fmt;

mod backtest;
mod binance;
mod errors;
mod factory;
mod models;
mod tardis;
mod ws;

use backtest::BacktestIngestor;
use binance::BinanceIngestor;

pub use factory::IngestorFactory;
pub use models::BinanceParser;
pub use tardis::*;

#[async_trait]
pub trait Ingestor {
    async fn start(&self);
}

#[derive(Clone)]
pub enum IngestorType {
    Backtest(BacktestIngestor),
    Binance(BinanceIngestor),
}

#[async_trait]
impl Ingestor for IngestorType {
    async fn start(&self) {
        match self {
            IngestorType::Backtest(b) => b.start().await,
            IngestorType::Binance(b) => b.start().await,
        }
    }
}

impl fmt::Display for IngestorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestorType::Backtest(_) => write!(f, "backtest"),
            IngestorType::Binance(_) => write!(f, "binance"),
        }
    }
}

#[derive(Clone)]
pub enum IngestorID {
    Backtest,
    Binance,
    Test,
}

impl fmt::Display for IngestorID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestorID::Backtest => write!(f, "backtest"),
            IngestorID::Binance => write!(f, "binance"),
            IngestorID::Test => write!(f, "test"),
        }
    }
}
