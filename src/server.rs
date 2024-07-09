use std::sync::Arc;

use tracing::{debug, error, info};

use crate::{
    config::GlobalConfig,
    ingestors::{factory::IngestorFactory, Ingestor, IngestorType},
    state::State,
};

pub struct Server {
    state: Arc<State>,
    ingestor_factory: IngestorFactory,
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

        // Wait for interrupt signal
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
    }

    async fn ingestor_task(state: Arc<State>, ingestors: Vec<IngestorType>) {
        info!("Spawning data provider task");

        info!("Providers starting...");
        let (tx, rx) = flume::unbounded();
        for provider in ingestors {
            let local_tx = tx.clone();
            tokio::spawn(async move { provider.start(local_tx).await });
        }
        info!("Providers started");

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
            ingestor_factory: IngestorFactory::new(&config.ingestors),
        }
    }
}
