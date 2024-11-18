use std::sync::Arc;

use crate::{config::StrategyAlgorithmConfig, Algorithm, CrossoverStrategyBuilder, StrategyConfig};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(config: &StrategyConfig) -> Vec<Arc<dyn Algorithm>> {
        let strategies: Vec<Arc<dyn Algorithm>> = config
            .strategies
            .iter()
            .map(|c| {
                let algo: Arc<dyn Algorithm> = match c {
                    StrategyAlgorithmConfig::Crossover(c) => Arc::new(
                        CrossoverStrategyBuilder::default()
                            .id(c.id.clone())
                            .price_source(c.price_spread_id.clone())
                            .volume_source(c.volume_spread_id.clone())
                            .build()
                            .expect("Failed to build CrossoverStrategy"),
                    ),
                    StrategyAlgorithmConfig::Spreader(_c) => unimplemented!(),
                };
                algo
            })
            .collect();

        strategies
    }
}
