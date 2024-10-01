use crate::{config::StrategyModuleConfig, crossover::CrossoverStrategy, manager::StrategyModule};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(configs: &[StrategyModuleConfig]) -> Vec<Box<dyn StrategyModule>> {
        let mut strategies = Vec::new();

        configs.iter().for_each(|c| {
            let strategy: Box<dyn StrategyModule> = match &c {
                StrategyModuleConfig::Crossover(c) => Box::new(CrossoverStrategy::from_config(c)),
            };
            strategies.push(strategy);
        });

        strategies
    }
}
