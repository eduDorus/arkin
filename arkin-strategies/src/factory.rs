use std::sync::Arc;

use crate::{config::StrategyModuleConfig, Algorithm, CrossoverStrategyBuilder};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(configs: &[StrategyModuleConfig]) -> Vec<Arc<dyn Algorithm>> {
        let mut strategies = Vec::new();

        configs.iter().for_each(|c| {
            let strategy: Arc<dyn Algorithm> = match &c {
                StrategyModuleConfig::Crossover(c) => Arc::new(
                    CrossoverStrategyBuilder::default()
                        .id(c.id.clone())
                        .price_source(c.price_spread_id.clone())
                        .volume_source(c.volume_spread_id.clone())
                        .build()
                        .expect("Failed to build Crossover Strategy"),
                ),
            };
            strategies.push(strategy);
        });

        strategies
    }
}
