use tracing::{error, info};

use crate::{
    config::GlobalConfig,
    data_providers::{factory::DataProviderFactory, DataProvider},
};

pub struct Server {
    data_provider_factory: DataProviderFactory,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(&self) {
        let (data_provider_tx, data_provider_rx) = flume::unbounded();

        info!("Providers starting...");
        for provider in &self.data_provider_factory.create_data_providers() {
            let local_provider = provider.clone();
            let local_tx = data_provider_tx.clone();
            tokio::spawn(async move { local_provider.start(local_tx).await });
        }
        info!("Providers started");

        info!("Listening to incoming");
        loop {
            let msg = data_provider_rx.recv_async().await;
            match msg {
                Ok(m) => info!(m),
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
            data_provider_factory: DataProviderFactory::new(&config.data_providers),
        }
    }
}
