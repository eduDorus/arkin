pub mod binance;
pub mod errors;
pub mod factory;
pub mod ws;

use core::fmt;

use binance::BinanceIngestor;

use crate::models::{OrderUpdate, PositionUpdate, Tick, Trade, TradeUpdate};

#[trait_variant::make(Send)]
pub trait Ingestor: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum DataEvent {
    // Market data
    Tick(Tick),
    Trade(Trade),
    AggTrade(Trade),

    // Account data
    PositionUpdate(PositionUpdate),
    OrderUpdate(OrderUpdate),
    TradeUpdate(TradeUpdate),
}

#[derive(Clone)]
pub enum IngestorType {
    Binance(BinanceIngestor),
}

impl Ingestor for IngestorType {
    async fn start(&self) {
        match self {
            IngestorType::Binance(b) => b.start().await,
        }
    }
}

impl fmt::Display for IngestorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestorType::Binance(_) => write!(f, "Binance"),
        }
    }
}
