use std::sync::Arc;

use crate::{config::IngestorConfig, state::State};

use super::{binance::BinanceIngestor, IngestorType};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn create_ingestors(state: Arc<State>, config: &[IngestorConfig]) -> Vec<IngestorType> {
        let mut ingestors = Vec::new();

        for config in config {
            match config {
                IngestorConfig::Binance(config) => {
                    ingestors.push(IngestorType::Binance(BinanceIngestor::new(state.to_owned(), config)));
                }
            }
        }

        ingestors
    }
}
