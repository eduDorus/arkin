use std::sync::Arc;

use tracing::{debug, error, info};

use crate::{
    config::GlobalConfig,
    features::{Feature, FeatureFactory, FeatureType},
    ingestors::{factory::IngestorFactory, Ingestor, IngestorType},
    state::State,
};

pub struct Server {
    state: Arc<State>,
    ingestor_factory: IngestorFactory,
    feature_factory: FeatureFactory,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(&self) {
        tokio::spawn(Server::ingestor_task(
            self.state.clone(),
            self.ingestor_factory.create_ingestors(),
        ));

        tokio::spawn(Server::feature_task(self.state.clone(), self.feature_factory.create_features()));

        // Wait for interrupt signal
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
    }

    async fn ingestor_task(state: Arc<State>, ingestors: Vec<IngestorType>) {
        info!("Spawning ingestor task");

        info!("Ingestors starting...");
        let (tx, rx) = flume::unbounded();
        for ingestor in ingestors {
            let local_tx = tx.clone();
            tokio::spawn(async move { ingestor.start(local_tx).await });
        }
        info!("Ingestors started");

        info!("Listening to incoming data");
        loop {
            let msg = rx.recv_async().await;
            match msg {
                Ok(m) => {
                    debug!("{}", m);
                    state.market_update(&m);
                }
                Err(e) => error!("{}", e),
            }
        }
    }

    async fn feature_task(state: Arc<State>, features: Vec<FeatureType>) {
        info!("Spawning feature task");

        info!("Features starting...");
        let (tx, rx) = flume::unbounded();
        for feature in features {
            info!("Starting feature {}", feature);
            let local_tx = tx.clone();
            tokio::spawn(async move { feature.start(local_tx).await });
        }
        info!("Features started");

        info!("Listening to created features");
        loop {
            let msg = rx.recv_async().await;
            match msg {
                Ok(m) => {
                    debug!("{}", m);
                    state.feature_update(&m);
                }
                Err(e) => error!("{}", e),
            }
        }
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
            ingestor_factory: IngestorFactory::new(config.ingestors.to_owned()),
            feature_factory: FeatureFactory::new(config.features.to_owned()),
        }
    }
}
