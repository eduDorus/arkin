use tracing::info;

use crate::config::IngestorFactoryConfig;

use super::{binance::BinanceIngestor, IngestorType};

pub struct IngestorFactory {
    config: IngestorFactoryConfig,
}

impl IngestorFactory {
    pub fn new(config: &IngestorFactoryConfig) -> IngestorFactory {
        IngestorFactory {
            config: config.to_owned(),
        }
    }

    pub fn create_ingestors(&self) -> Vec<IngestorType> {
        let mut ingestors = Vec::new();

        for (name, config) in &self.config.binance {
            if config.enabled {
                info!("Creating Binance data provider: {}", name);
                ingestors.push(IngestorType::Binance(BinanceIngestor::new(config)));
            }
        }

        ingestors
    }
}
