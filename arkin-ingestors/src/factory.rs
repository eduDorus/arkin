use std::{sync::Arc, time::Duration};

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use time::OffsetDateTime;

use crate::{
    config::{IngestorConfig, IngestorsConfig},
    BinanceIngestor, IngestorError, SimIngestor, TardisIngestor,
};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn from_config(
        config: &IngestorsConfig,
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
    ) -> Vec<Arc<dyn RunnableService<Error = IngestorError>>> {
        config
            .ingestors
            .iter()
            .map(|config| {
                let ingestor: Arc<dyn RunnableService<Error = IngestorError>> = match config {
                    IngestorConfig::Binance(c) => Self::binance_ingestor(
                        pubsub.clone(),
                        persistence.clone(),
                        c.ws_url.to_owned(),
                        c.ws_channels.to_owned(),
                        c.api_key.to_owned(),
                        c.api_secret.to_owned(),
                        c.connections_per_manager,
                        c.duplicate_lookback,
                    ),
                    IngestorConfig::Tardis(c) => {
                        Arc::new(TardisIngestor::from_config(c, pubsub.clone(), persistence.clone()))
                    }
                };
                ingestor
            })
            .collect()
    }

    pub fn binance_ingestor(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        ws_url: String,
        ws_channels: Vec<String>,
        api_key: Option<String>,
        api_secret: Option<String>,
        connections_per_manager: usize,
        duplicate_lookback: usize,
    ) -> Arc<dyn RunnableService<Error = IngestorError>> {
        Arc::new(
            BinanceIngestor::builder()
                .pubsub(pubsub)
                .persistence(persistence)
                .url(ws_url.parse().expect("Failed to parse ws binance URL"))
                .channels(ws_channels)
                .api_key(api_key)
                .api_secret(api_secret)
                .connections_per_manager(connections_per_manager)
                .duplicate_lookback(duplicate_lookback)
                .build(),
        )
    }

    pub fn create_simulation_ingestor(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        instruments: Vec<Arc<Instrument>>,
        tick_frequency: Duration,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Arc<dyn RunnableService<Error = IngestorError>> {
        Arc::new(
            SimIngestor::builder()
                .pubsub(pubsub)
                .persistence(persistence)
                .instruments(instruments)
                .tick_frequency(tick_frequency)
                .start(start)
                .end(end)
                .build(),
        )
    }
}
