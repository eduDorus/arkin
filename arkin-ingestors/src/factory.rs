use std::sync::Arc;

use arkin_persistance::PersistanceService;

use crate::{config::IngestorModuleConfig, service::Ingestor};

use super::binance::BinanceIngestor;

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        configs: &[IngestorModuleConfig],
        persistance_service: Arc<PersistanceService>,
    ) -> Vec<Box<dyn Ingestor>> {
        configs
            .iter()
            .map(|config| {
                let ingestor: Box<dyn Ingestor> = match config {
                    IngestorModuleConfig::Binance(c) => Box::new(BinanceIngestor::new(c, persistance_service.clone())),
                    IngestorModuleConfig::Tardis(_) => unimplemented!(),
                };
                ingestor
            })
            .collect()
    }
}
