use std::sync::Arc;

use tracing::info;

use crate::{
    allocation::{Allocation, AllocationFactory, AllocationType},
    config::GlobalConfig,
    execution::{Execution, ExecutionFactory, ExecutionType},
    features::{Feature, FeatureFactory, FeatureType},
    ingestors::{Ingestor, IngestorFactory, IngestorType},
    state::StateManager,
    strategies::{Strategy, StrategyFactory, StrategyType},
};

pub struct Server {
    state: Arc<StateManager>,
    config: GlobalConfig,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn run(&self) {
        let ingestors = IngestorFactory::from_config(self.state.clone(), &self.config.ingestors);
        tokio::spawn(Server::ingestor_task(ingestors));

        let features = FeatureFactory::from_config(self.state.clone(), &self.config.features);
        tokio::spawn(Server::feature_task(features));

        let strategies = StrategyFactory::from_config(self.state.clone(), &self.config.strategies);
        tokio::spawn(Server::strategy_task(strategies));

        let allocation = AllocationFactory::from_config(self.state.clone(), &self.config.allocation);
        tokio::spawn(Server::allocation_task(allocation));

        let execution = ExecutionFactory::from_config(self.state.clone(), &self.config.execution);
        tokio::spawn(Server::execution_task(execution));

        // Wait for interrupt signal
        tokio::signal::ctrl_c().await.expect("Failed to listen for event");
    }

    async fn ingestor_task(ingestors: Vec<IngestorType>) {
        info!("Spawning ingestor tasks...");
        for ingestor in ingestors {
            tokio::spawn(async move { ingestor.start().await });
        }
    }

    async fn feature_task(features: Vec<FeatureType>) {
        info!("Spawning feature tasks...");
        for feature in features {
            tokio::spawn(async move { feature.start().await });
        }
    }

    async fn strategy_task(strategies: Vec<StrategyType>) {
        info!("Spawning trader tasks...");
        for strategy in strategies {
            tokio::spawn(async move { strategy.start().await });
        }
    }

    async fn allocation_task(allocation: AllocationType) {
        info!("Spawning allocation tasks...");
        tokio::spawn(async move { allocation.start().await });
    }

    async fn execution_task(executors: Vec<ExecutionType>) {
        info!("Spawning execution tasks...");
        for executor in executors {
            tokio::spawn(async move { executor.start().await });
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
            state: Arc::new(StateManager::new(&config.state)),
            config,
        }
    }
}
