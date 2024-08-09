use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::info;

use crate::{
    config::BacktestIngestorConfig,
    ingestors::IngestorID,
    models::{Event, Instrument, Trade, Venue},
    state::StateManager,
};

use super::Ingestor;

#[derive(Clone)]
#[allow(unused)]
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

#[async_trait]
impl Ingestor for BacktestIngestor {
    async fn start(&self) {
        info!("Starting backtest ingestor...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        let mut trade_id = 0;

        loop {
            interval.tick().await;
            let trade = Trade::new(
                OffsetDateTime::now_utc(),
                OffsetDateTime::now_utc(),
                Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
                trade_id,
                Decimal::new(50000, 0).into(),
                Decimal::new(1, 0).into(),
                IngestorID::Backtest,
            );
            self.state.add_event(Event::Trade(trade));
            trade_id += 1;
        }
    }
}
