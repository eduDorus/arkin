use std::{sync::Arc, time::Duration};

use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_persistence::prelude::*;
use tracing::error;

use crate::{
    cli::{DownloadArgs, IngestorType, IngestorsArgs, InsightsArgs, SimulationArgs},
    config::IngestorsConfig,
};

pub enum IngestorTypes {
    Binance,
    Tardis,
    Simulation,
}

pub struct IngestorFactory {}

impl IngestorFactory {
    pub async fn init(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        args: &IngestorsArgs,
    ) -> Arc<dyn RunnableService> {
        let config = load::<IngestorsConfig>();
        match args.ingestor {
            IngestorType::Binance => {
                let c = config.ingestors.binance.expect("Binance config not found");

                // Create channels for websocket
                let mut channels = vec![];
                for channel in &c.channels {
                    for instrument in &args.instruments {
                        channels.push(format!("{}@{}", instrument.to_lowercase(), channel));
                    }
                }
                Arc::new(
                    BinanceIngestor::builder()
                        .pubsub(pubsub.publisher().await)
                        .persistence(persistence)
                        .url(c.ws_url.parse().expect("Failed to parse ws binance URL"))
                        .channels(channels)
                        .api_key(c.api_key.clone())
                        .api_secret(c.api_secret.clone())
                        .connections_per_manager(c.connections_per_manager)
                        .duplicate_lookback(c.duplicate_lookback)
                        .build(),
                )
            }
        }
    }

    pub async fn init_download(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        args: &DownloadArgs,
    ) -> Arc<dyn RunnableService> {
        let config = load::<IngestorsConfig>();
        let c = config.ingestors.tardis.expect("Tardis config not found");
        let client = TardisHttpClient::builder()
            .api_secret(c.api_secret.clone())
            .base_url(c.http_url.clone())
            .build();

        let ingestor = TardisIngestor::builder()
            .pubsub(pubsub.handle().await)
            .persistence(persistence)
            .client(client)
            .venue(args.venue.clone())
            .channel(args.channel.clone())
            .start(args.start)
            .end(args.end)
            .instruments(args.instruments.clone())
            .max_concurrent_requests(c.max_concurrent_requests)
            .build();
        Arc::new(ingestor)
    }

    pub async fn init_insights(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        args: &InsightsArgs,
    ) -> Arc<dyn RunnableService> {
        // Load Instruments
        let mut instruments = vec![];
        for symbol in &args.instruments {
            match persistence.instrument_store.read_by_venue_symbol(symbol).await {
                Ok(instr) => instruments.push(instr),
                Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
            }
        }
        Arc::new(
            SimIngestor::builder()
                .pubsub(pubsub.publisher().await)
                .persistence(persistence)
                .instruments(instruments)
                .tick_frequency(Duration::from_secs(args.tick_frequency))
                .start(args.start)
                .end(args.end)
                .build(),
        )
    }

    pub async fn init_simulation(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        args: &SimulationArgs,
    ) -> Arc<dyn RunnableService> {
        // Load Instruments
        let mut instruments = vec![];
        for symbol in &args.instruments {
            match persistence.instrument_store.read_by_venue_symbol(symbol).await {
                Ok(instr) => instruments.push(instr),
                Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
            }
        }
        Arc::new(
            SimIngestor::builder()
                .pubsub(pubsub.publisher().await)
                .persistence(persistence)
                .instruments(instruments)
                .tick_frequency(Duration::from_secs(args.tick_frequency))
                .start(args.start)
                .end(args.end)
                .build(),
        )
    }
}
