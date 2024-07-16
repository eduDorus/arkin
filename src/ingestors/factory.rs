use std::sync::Arc;

use crate::{config::IngestorConfig, state::StateManager};

use super::{backtest::BacktestIngestor, binance::BinanceIngestor, IngestorType};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn create_ingestors(state: Arc<StateManager>, config: &[IngestorConfig]) -> Vec<IngestorType> {
        let mut ingestors = Vec::new();

        for config in config {
            match config {
                IngestorConfig::Backtest(config) => {
                    ingestors.push(IngestorType::Backtest(BacktestIngestor::new(state.to_owned(), config)))
                }
                IngestorConfig::Binance(config) => {
                    ingestors.push(IngestorType::Binance(BinanceIngestor::new(state.to_owned(), config)));
                }
            }
        }

        ingestors
    }
}
