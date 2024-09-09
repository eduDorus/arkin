use crate::{config::IngestorModuleConfig, manager::IngestorModule};

use super::binance::BinanceIngestor;

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(configs: &[IngestorModuleConfig]) -> Vec<Box<dyn IngestorModule>> {
        configs
            .iter()
            .map(|config| {
                let ingestor: Box<dyn IngestorModule> = match config {
                    IngestorModuleConfig::Binance(c) => Box::new(BinanceIngestor::new(c)),
                    IngestorModuleConfig::Tardis(_) => unimplemented!(),
                };
                ingestor
            })
            .collect()
    }
}
