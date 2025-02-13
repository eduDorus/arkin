use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_strategies::{CrossoverStrategy, StrategyService};

use crate::config::{StrategyAlgorithmConfig, StrategyConfig};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn init(pubsub: Arc<PubSub>) -> Vec<Arc<dyn StrategyService>> {
        let config = load::<StrategyConfig>();
        let strategies: Vec<Arc<dyn StrategyService>> = config
            .strategies
            .iter()
            .map(|c| {
                let algo: Arc<dyn StrategyService> = match c {
                    StrategyAlgorithmConfig::Crossover(c) => Arc::new(
                        CrossoverStrategy::builder()
                            .pubsub(pubsub.clone())
                            .id(test_strategy())
                            .fast_ma(c.fast_ma.clone())
                            .slow_ma(c.slow_ma.clone())
                            .build(),
                    ),
                    StrategyAlgorithmConfig::Spreader(_c) => unimplemented!(),
                };
                algo
            })
            .collect();

        strategies
    }
}
