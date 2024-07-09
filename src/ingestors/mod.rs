pub mod binance;
pub mod errors;
pub mod factory;
pub mod ws;

use binance::BinanceIngestor;
use flume::Sender;

use crate::models::{MarketEvent, OrderUpdate, PositionUpdate, Tick, Trade, TradeUpdate};

#[trait_variant::make(Send)]
pub trait Ingestor: Clone {
    async fn start(&self, sender: Sender<MarketEvent>);
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
    async fn start(&self, sender: Sender<MarketEvent>) {
        match self {
            IngestorType::Binance(b) => b.start(sender).await,
        }
    }
}
