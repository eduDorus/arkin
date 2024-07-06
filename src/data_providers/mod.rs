pub mod binance;
pub mod errors;
pub mod factory;
pub mod ws;

use binance::BinanceDataProvider;
use flume::Sender;

use crate::models::{MarketEvent, OrderUpdate, PositionUpdate, Tick, Trade, TradeUpdate};

#[trait_variant::make(Send)]
pub trait DataProvider: Clone {
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
pub enum DataProviderType {
    Binance(BinanceDataProvider),
}

impl DataProvider for DataProviderType {
    async fn start(&self, sender: Sender<MarketEvent>) {
        match self {
            DataProviderType::Binance(b) => b.start(sender).await,
        }
    }
}
