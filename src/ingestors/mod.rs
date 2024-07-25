mod backtest;
mod binance;
mod errors;
mod factory;
mod ws;

use core::fmt;

use backtest::BacktestIngestor;
use binance::BinanceIngestor;

pub use factory::IngestorFactory;

#[trait_variant::make(Send)]
pub trait Ingestor {
    async fn start(&self);
}

#[derive(Clone)]
pub enum IngestorType {
    Backtest(BacktestIngestor),
    Binance(BinanceIngestor),
}

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
            IngestorType::Backtest(_) => write!(f, "Backtest"),
            IngestorType::Binance(_) => write!(f, "Binance"),
        }
    }
}

#[derive(Clone)]
pub enum IngestorID {
    Backtest,
    Binance,
    Tardis,
}

impl fmt::Display for IngestorID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestorID::Backtest => write!(f, "backtest"),
            IngestorID::Binance => write!(f, "binance"),
            IngestorID::Tardis => write!(f, "tardis"),
        }
    }
}
