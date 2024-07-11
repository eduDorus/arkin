use crate::config::IngestorConfig;

use super::{binance::BinanceIngestor, IngestorType};

pub struct IngestorFactory {
    config: Vec<IngestorConfig>,
}

impl IngestorFactory {
    pub fn new(config: Vec<IngestorConfig>) -> IngestorFactory {
        IngestorFactory { config }
    }

    pub fn create_ingestors(&self) -> Vec<IngestorType> {
        let mut ingestors = Vec::new();

        for config in &self.config {
            match config {
                IngestorConfig::Binance(config) => {
                    ingestors.push(IngestorType::Binance(BinanceIngestor::new(config)));
                }
            }
        }

        ingestors
    }
}
