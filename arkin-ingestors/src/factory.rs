use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use time::OffsetDateTime;

use crate::{
    config::{IngestorConfig, IngestorsConfig},
    traits::Ingestor,
    BinanceIngestor, SimIngestor, TardisIngestor,
};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        config: &IngestorsConfig,
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
    ) -> Vec<Arc<dyn Ingestor>> {
        config
            .ingestors
            .iter()
            .map(|config| {
                let ingestor: Arc<dyn Ingestor> = match config {
                    IngestorConfig::Binance(c) => Arc::new(
                        BinanceIngestor::builder()
                            .pubsub(pubsub.clone())
                            .persistence(persistence.clone())
                            .url(c.ws_url.parse().expect("Failed to parse ws binance URL"))
                            .channels(c.ws_channels.to_owned())
                            .api_key(c.api_key.to_owned())
                            .api_secret(c.api_secret.to_owned())
                            .connections_per_manager(c.connections_per_manager)
                            .duplicate_lookback(c.duplicate_lookback)
                            .build(),
                    ),
                    IngestorConfig::Tardis(c) => {
                        Arc::new(TardisIngestor::from_config(c, pubsub.clone(), persistence.clone()))
                    }
                };
                ingestor
            })
            .collect()
    }

    pub fn create_simulation_ingestor(
        persistence: Arc<PersistenceService>,
        instruments: Vec<Arc<Instrument>>,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Arc<dyn Ingestor> {
        Arc::new(
            SimIngestor::builder()
                .persistence(persistence)
                .instruments(instruments)
                .start(start)
                .end(end)
                .build(),
        )
    }
}
