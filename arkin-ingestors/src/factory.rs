use std::sync::Arc;

use arkin_persistence::prelude::*;

use crate::{binance::BinanceIngestorBuilder, config::IngestorModuleConfig, traits::Ingestor, TardisIngestor};

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
                    IngestorModuleConfig::Binance(c) => Box::new(
                        BinanceIngestorBuilder::default()
                            .persistence_service(persistence_service.clone())
                            .url(c.ws_url.parse().expect("Failed to parse ws binance URL"))
                            .channels(c.ws_channels.to_owned())
                            .api_key(c.api_key.to_owned())
                            .api_secret(c.api_secret.to_owned())
                            .connections_per_manager(c.connections_per_manager)
                            .duplicate_lookback(c.duplicate_lookback)
                            .build()
                            .expect("Failed to build BinanceIngestor"),
                    ),
                    IngestorModuleConfig::Tardis(c) => {
                        Box::new(TardisIngestor::from_config(c, persistence_service.clone()))
                    }
                };
                ingestor
            })
            .collect()
    }
}
