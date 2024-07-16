use std::sync::Arc;

use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::info;

use crate::{
    config::BacktestIngestorConfig,
    models::{Asset, Instrument, MarketEvent, PerpetualContract, Price, Quantity, Trade, Venue},
    state::StateManager,
};

use super::Ingestor;

#[derive(Clone)]
pub struct BacktestIngestor {
    state: Arc<StateManager>,
    market_data: bool,
}

impl BacktestIngestor {
    pub fn new(state: Arc<StateManager>, config: &BacktestIngestorConfig) -> Self {
        BacktestIngestor {
            state,
            market_data: config.market_data,
        }
    }
}

impl Ingestor for BacktestIngestor {
    async fn start(&self) {
        info!("Starting backtest ingestor...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            let perp = PerpetualContract::new(&Venue::Binance, &Asset::new("BTC"), &Asset::new("USDT"));
            let trade = Trade::new(
                Instrument::Perpetual(perp),
                OffsetDateTime::now_utc(),
                Price::new(Decimal::new(50000, 0)).unwrap(),
                Quantity::new(Decimal::new(1, 0)),
            );
            self.state.market_update(&MarketEvent::Trade(trade))
        }
    }
}
