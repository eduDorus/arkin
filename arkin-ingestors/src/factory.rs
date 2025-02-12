use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use time::OffsetDateTime;

use crate::{
    config::IngestorConfig,
    tardis::{TardisChannel, TardisExchange, TardisHttpClient},
    BinanceIngestor, IngestorError, SimIngestor, TardisIngestor,
};

pub struct IngestorFactory {}

impl IngestorFactory {
    pub fn init(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        config: &IngestorConfig,
        args: &IngestorsCommands,
    ) -> Result<Arc<dyn RunnableService>, IngestorError> {
        match args {
            IngestorsCommands::Binance(a) => {
                if let Some(c) = &config.binance {
                    let ingestor = BinanceIngestor::builder()
                        .pubsub(pubsub)
                        .persistence(persistence)
                        .url(c.ws_url.parse().expect("Failed to parse ws binance URL"))
                        .channels(a.channels.clone())
                        .api_key(c.api_key.clone())
                        .api_secret(c.api_secret.clone())
                        .connections_per_manager(c.connections_per_manager)
                        .duplicate_lookback(c.duplicate_lookback)
                        .build();
                    Ok(Arc::new(ingestor))
                } else {
                    Err(IngestorError::ConfigError("Binance ingestor config not found".to_string()))
                }
            }
            IngestorsCommands::Tardis(a) => {
                if let Some(c) = &config.tardis {
                    let client = TardisHttpClient::builder()
                        .api_secret(c.api_secret.clone())
                        .base_url(c.http_url.clone())
                        .build();

                    let ingestor = TardisIngestor::builder()
                        .pubsub(pubsub)
                        .persistence(persistence)
                        .client(client)
                        .venue(TardisExchange::from_str(&a.venue).expect("Invalid venue for tardis"))
                        .channel(TardisChannel::from_str(&a.channel).expect("Invalid channel for tardis"))
                        .start(a.start)
                        .end(a.end)
                        .instruments(a.instruments.clone())
                        .max_concurrent_requests(c.max_concurrent_requests)
                        .build();
                    Ok(Arc::new(ingestor))
                } else {
                    Err(IngestorError::ConfigError("Tardis ingestor config not found".to_string()))
                }
            }
        }
    }

    pub fn init_simulation(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        instruments: Vec<Arc<Instrument>>,
        tick_frequency: Duration,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Arc<dyn RunnableService> {
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

    pub fn binance_ingestor(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        ws_url: String,
        ws_channels: Vec<String>,
        api_key: Option<String>,
        api_secret: Option<String>,
        connections_per_manager: usize,
        duplicate_lookback: usize,
    ) -> Arc<dyn RunnableService> {
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
}
