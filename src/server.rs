use std::sync::Arc;

use tracing::{info};

use crate::{
    config::GlobalConfig,
    features::{Feature, FeatureFactory, FeatureType},
    ingestors::{factory::IngestorFactory, Ingestor, IngestorType},
    state::State,
    trader::{Trader, TraderFactory, TraderType},
};

pub struct Server {
    state: Arc<State>,
    config: GlobalConfig,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(&self) {
        let ingestors = IngestorFactory::create_ingestors(self.state.clone(), &self.config.ingestors);
        tokio::spawn(Server::ingestor_task(ingestors));

        let features = FeatureFactory::create_features(self.state.clone(), &self.config.features);
        tokio::spawn(Server::feature_task(features));

        let traders = TraderFactory::create_traders(self.state.clone(), &self.config.traders);
        tokio::spawn(async move { Server::trader_task(traders) });

        // Wait for interrupt signal
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
    }

    async fn ingestor_task(ingestors: Vec<IngestorType>) {
        info!("Spawning ingestor task");

        info!("Ingestors starting...");
        for ingestor in ingestors {
            info!("Starting ingestor {}", ingestor);
            tokio::spawn(async move { ingestor.start().await });
        }
        info!("Ingestors spawned");
    }

    async fn feature_task(features: Vec<FeatureType>) {
        info!("Spawning feature task");

        info!("Features starting...");
        for feature in features {
            info!("Starting feature {}", feature);
            tokio::spawn(async move { feature.start().await });
        }
        info!("Features spawned");
    }

    async fn trader_task(traders: Vec<TraderType>) {
        info!("Spawning trader task");

        info!("Traders starting...");
        for trader in traders {
            info!("Starting trader {}", trader);
            tokio::spawn(async move { trader.start().await });
        }
        info!("Traders spawned");
    }
}

#[derive(Default)]
pub struct ServerBuilder {
    config: Option<GlobalConfig>,
}

impl ServerBuilder {
    pub fn config(mut self, config: &GlobalConfig) -> Self {
        self.config = Some(config.to_owned());
        self
    }

    pub fn build(self) -> Server {
        let config = self.config.unwrap();
        Server {
            state: Arc::new(State::new(&config.state)),
            config,
        }
    }
}
