use std::{str::FromStr, sync::Arc};

use arkin_cli::{Cli, Commands};
use arkin_ingestor_tardis::{TardisConfig, TardisIngestor};
use arkin_persistence::{Persistence, PersistenceConfig};
use clap::Parser;
use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::info;

use arkin_core::prelude::*;
use uuid::Uuid;

#[tokio::main(flavor = "current_thread")]
// #[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    info!("Starting arkin 🚀");

    let args = Cli::parse();
    info!("args: {:?}", args);

    match args.command {
        Commands::Download(a) => {
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
            let persistence = Persistence::new(&config, instance, false, false, false);
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
        _ => todo!(),
    }
}

// let audit = Audit::new("audit");
// let audit_service = Service::new(audit, Some(pubsub.subscribe(EventFilter::All)));
