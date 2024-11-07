use std::sync::Arc;

use arkin_persistence::prelude::*;

use crate::{config::IngestorModuleConfig, service::Ingestor, TardisIngestor};

use super::binance::BinanceIngestor;

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        configs: &[IngestorModuleConfig],
        persistence_service: Arc<PersistenceService>,
    ) -> Vec<Box<dyn Ingestor + Send + Sync>> {
        configs
            .iter()
            .map(|config| {
                let ingestor: Box<dyn Ingestor + Send + Sync> = match config {
                    IngestorModuleConfig::Binance(c) => {
                        Box::new(BinanceIngestor::from_config(c, persistence_service.clone()))
                    }
                    IngestorModuleConfig::Tardis(c) => {
                        Box::new(TardisIngestor::from_config(c, persistence_service.clone()))
                    }
                };
                ingestor
            })
            .collect()
    }
}
