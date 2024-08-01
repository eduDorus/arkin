use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::info;

use crate::{
    config::BacktestIngestorConfig,
    ingestors::IngestorID,
    models::{Asset, Event, Instrument, PerpetualContract, Price, Quantity, Trade, Venue},
    state::State,
};

use super::Ingestor;

#[derive(Clone)]
#[allow(unused)]
pub struct BacktestIngestor {
    state: Arc<State>,
    market_data: bool,
}

impl BacktestIngestor {
    pub fn new(state: Arc<State>, config: &BacktestIngestorConfig) -> Self {
        BacktestIngestor {
            state,
            market_data: config.market_data,
        }
    }
}

#[async_trait]
impl Ingestor for BacktestIngestor {
    async fn start(&self) {
        info!("Starting backtest ingestor...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        let mut trade_id = 0;

        loop {
            interval.tick().await;
            let perp = PerpetualContract::new(&Venue::Binance, &Asset::new("BTC"), &Asset::new("USDT"));
            let trade = Trade::new(
                Instrument::Perpetual(perp),
                OffsetDateTime::now_utc(),
                trade_id,
                Price::new(Decimal::new(50000, 0)).unwrap(),
                Quantity::new(Decimal::new(1, 0)),
                IngestorID::Backtest,
            );
            self.state.add_event(Event::Trade(trade));
            trade_id += 1;
        }
    }
}
