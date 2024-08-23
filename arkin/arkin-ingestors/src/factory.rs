use std::sync::Arc;

use crate::{config::IngestorConfig, state::StateManager};

use super::{backtest::BacktestIngestor, binance::BinanceIngestor, IngestorType};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(state: Arc<StateManager>, config: &[IngestorConfig]) -> Vec<IngestorType> {
        let mut ingestors = Vec::new();

        for config in config {
            let ingestor = match config {
                IngestorConfig::Backtest(c) => IngestorType::Backtest(BacktestIngestor::new(state.to_owned(), c)),
                IngestorConfig::Binance(c) => IngestorType::Binance(BinanceIngestor::new(state.to_owned(), c)),
            };
            ingestors.push(ingestor);
        }

        ingestors
    }
}
