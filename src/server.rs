use std::sync::Arc;

use tracing::{error, info};

use crate::{
    config::GlobalConfig,
    data_providers::{factory::DataProviderFactory, DataProvider, DataProviderType},
    state::State,
};

pub struct Server {
    state: Arc<State>,
    data_provider_factory: DataProviderFactory,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(&self) {
        tokio::spawn(Server::data_provider_task(
            self.state.clone(),
            self.data_provider_factory.create_data_providers(),
        ));

        // Wait for interrupt signal
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
    }

    async fn data_provider_task(_state: Arc<State>, data_providers: Vec<DataProviderType>) {
        info!("Spawning data provider task");

        info!("Providers starting...");
        let (tx, rx) = flume::unbounded();
        for provider in data_providers {
            let local_tx = tx.clone();
            tokio::spawn(async move { provider.start(local_tx).await });
        }
        info!("Providers started");

        info!("Listening to incoming data");
        loop {
            let msg = rx.recv_async().await;
            match msg {
                Ok(m) => info!(m), // Would update the state here
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
            data_provider_factory: DataProviderFactory::new(&config.data_providers),
        }
    }
}
