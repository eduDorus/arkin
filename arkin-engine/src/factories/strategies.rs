use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_strategies::prelude::*;

use crate::config::{StrategyAlgorithmConfig, StrategyConfig};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub async fn init(pubsub: Arc<PubSub>, persistence: Arc<PersistenceService>) -> Vec<Arc<dyn StrategyService>> {
        let config = load::<StrategyConfig>();
        let mut strategies: Vec<Arc<dyn StrategyService>> = vec![];
        for c in config.strategies {
            let algo: Arc<dyn StrategyService> = match c {
                StrategyAlgorithmConfig::Crossover(c) => {
                    let strategy = persistence
                        .strategy_store
                        .read_by_name_or_create(&c.name)
                        .await
                        .expect("Failed to read or create strategy");

                    Arc::new(
                        CrossoverStrategy::builder()
                            .pubsub(pubsub.handle("CorssoverStrategy").await)
                            .strategy(strategy)
                            .fast_ma(c.fast_ma.clone())
                            .slow_ma(c.slow_ma.clone())
                            .build(),
                    )
                }
                StrategyAlgorithmConfig::Spreader(_c) => unimplemented!(),
                StrategyAlgorithmConfig::Forecast(c) => {
                    let strategy = persistence
                        .strategy_store
                        .read_by_name_or_create(&c.name)
                        .await
                        .expect("Failed to read or create strategy");

                    Arc::new(
                        ForecastStrategy::builder()
                            .pubsub(pubsub.handle("ForecastStrategy").await)
                            .strategy(strategy)
                            .inputs(c.inputs.clone())
                            .threshold(c.threshold)
                            .build(),
                    )
                }
                StrategyAlgorithmConfig::Agent(c) => {
                    let strategy = persistence
                        .strategy_store
                        .read_by_name_or_create(&c.name)
                        .await
                        .expect("Failed to read or create strategy");

                    Arc::new(
                        AgentStrategy::builder()
                            .pubsub(pubsub.handle("AgentStrategy").await)
                            .strategy(strategy)
                            .model_location(c.model_location.clone())
                            .model_name(c.model_name.clone())
                            .model_version(c.model_version.clone())
                            .action_space(c.action_space)
                            .n_layers(c.n_layers)
                            .hidden_size(c.hidden_size)
                            .inputs(c.inputs.clone())
                            .build(),
                    )
                }
            };
            strategies.push(algo);
        }
        strategies
    }
}
