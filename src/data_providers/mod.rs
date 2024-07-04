use flume::Sender;

use crate::models::MarketEvent;

pub mod binance;
pub mod errors;
pub mod ws;

pub enum DataProviderType {
    Binance,
    WebSocket,
}

#[trait_variant::make(Send)]
pub trait DataProvider {
    async fn start(&self, sender: Sender<MarketEvent>);
}
