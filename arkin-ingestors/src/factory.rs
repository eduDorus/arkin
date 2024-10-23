use std::sync::Arc;

use arkin_persistance::PersistanceService;

use crate::{config::IngestorModuleConfig, service::Ingestor, TardisIngestor};

use super::binance::BinanceIngestor;

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        configs: &[IngestorModuleConfig],
        persistance_service: Arc<PersistanceService>,
    ) -> Vec<Arc<Box<dyn Ingestor>>> {
        configs
            .iter()
            .map(|config| {
                let ingestor: Box<dyn Ingestor> = match config {
                    IngestorModuleConfig::Binance(c) => {
                        Box::new(BinanceIngestor::from_config(c, persistance_service.clone()))
                    }
                    IngestorModuleConfig::Tardis(c) => {
                        Box::new(TardisIngestor::from_config(c, persistance_service.clone()))
                    }
                };
                Arc::new(ingestor)
            })
            .collect()
    }
}
