use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_cli::{Cli, Commands};
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_ingestor_tardis::{TardisConfig, TardisIngestor};
use arkin_insights::{prelude::InsightsConfig, Insights};
use arkin_persistence::{Persistence, PersistenceConfig};
use clap::Parser;
use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::{error, info};

use arkin_core::prelude::*;
use uuid::Uuid;

#[tokio::main(flavor = "current_thread")]
// #[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    let args = Cli::parse();
    info!("args: {:?}", args);

    match args.command {
        Commands::Download(a) => {
            info!("Starting arkin downloader 🚀");
            let time = LiveSystemTime::new();

            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
                .name("downloader".to_owned())
                .instance_type(InstanceType::Utility)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
            // let persistence_service = Service::new(persistence.to_owned(), None);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

            let cfg = load::<TardisConfig>();
            let ingestor = Arc::new(
                TardisIngestor::builder()
                    .publisher(pubsub.publisher())
                    .persistence(persistence)
                    .venue(a.venue.clone())
                    .channel(a.channel.clone())
                    .start(a.start)
                    .end(a.end)
                    .instruments(a.instruments.clone())
                    .max_concurrent_requests(cfg.tardis.max_concurrent_requests)
                    .base_url(cfg.tardis.http_url)
                    .api_secret(cfg.tardis.api_secret)
                    .build(),
            );
            let download_service = Service::new(ingestor, None);

            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 10).await;
            engine.register(download_service, 0, 10).await;
            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Ingestor(a) => {
            info!("Starting arkin ingestor 🚀");
            let time = LiveSystemTime::new();

            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
                .name("downloader".to_owned())
                .instance_type(InstanceType::Utility)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
            // let persistence_service = Service::new(persistence.to_owned(), None);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 10).await;
            // engine.register(download_service, 0, 10).await;
            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Insights(a) => {
            info!("Starting arkin insights 🚀");

            // Start and end time
            let start_time = a.start;
            let end_time = a.end;

            let time = MockTime::new_from(start_time, a.tick_frequency);

            // Init pubsub
            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            // Init persistence
            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("b787c86a-aff3-4495-b898-008f0fde633c").unwrap())
                .name("insights".to_owned())
                .instance_type(InstanceType::Insights)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, a.only_normalized, a.only_predictions, a.dry_run);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Insights)));

            let mut instruments = Vec::new();
            for inst in a.instruments {
                match persistence.get_instrument_by_venue_symbol(&inst).await {
                    Ok(i) => instruments.push(i),
                    Err(e) => panic!("{}", e),
                }
            }

            // Init sim ingestor
            let binance_ingestor = Arc::new(
                SimBinanceIngestor::builder()
                    .identifier("sim-binance-ingestor".to_owned())
                    ._time(time.to_owned())
                    .start(start_time)
                    .end(end_time + Duration::from_secs(3600))
                    .instruments(instruments.clone())
                    .persistence(persistence.to_owned())
                    .publisher(pubsub.publisher())
                    .build(),
            );
            let binance_ingestor_service = Service::new(binance_ingestor, None);

            // Insights service
            let pipeline_config = load::<InsightsConfig>();
            let pipeline_info: Arc<Pipeline> = Pipeline::builder()
                .id(Uuid::from_str("f031d4e2-2ada-4651-83fa-aef515accb29").unwrap())
                .name(a.pipeline)
                .description("Pipeline version v1.6.0 (Multi Asset)".to_owned())
                .created(time.now().await)
                .updated(time.now().await)
                .build()
                .into();
            if let Err(e) = persistence.insert_pipeline(pipeline_info.clone()).await {
                error!("{}", e);
            }
            let insights = Insights::new(
                pubsub.publisher(),
                pipeline_info,
                &pipeline_config.insights_service.pipeline,
                instruments,
                a.warmup,
            )
            .await;
            let insights_service = Service::new(
                insights,
                Some(pubsub.subscribe(EventFilter::Events(vec![
                    EventType::AggTradeUpdate,
                    EventType::TickUpdate,
                    EventType::InsightsTick,
                ]))),
            );

            // Setup engine
            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 9).await;
            engine.register(insights_service, 0, 8).await;
            engine.register(binance_ingestor_service, 1, 7).await;

            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        _ => todo!(),
    }
}

// let audit = Audit::new("audit");
// let audit_service = Service::new(audit, Some(pubsub.subscribe(EventFilter::All)));
