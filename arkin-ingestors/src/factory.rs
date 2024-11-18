use std::sync::Arc;

use arkin_persistence::prelude::*;

use crate::{
    binance::BinanceIngestorBuilder,
    config::{IngestorConfig, IngestorsConfig},
    traits::Ingestor,
    TardisIngestor,
};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        config: &IngestorsConfig,
        persistence_service: Arc<PersistenceService>,
    ) -> Vec<Arc<dyn Ingestor>> {
        config
            .ingestors
            .iter()
            .map(|config| {
                let ingestor: Arc<dyn Ingestor> = match config {
                    IngestorConfig::Binance(c) => Arc::new(
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
                    IngestorConfig::Tardis(c) => Arc::new(TardisIngestor::from_config(c, persistence_service.clone())),
                };
                ingestor
            })
            .collect()
    }
}
