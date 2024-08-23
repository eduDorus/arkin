use crate::{config::StrategyConfig, crossover::CrossoverStrategy, manager::StrategyModule};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(configs: &[StrategyConfig]) -> Vec<Box<dyn StrategyModule>> {
        let mut strategies = Vec::new();

        configs.iter().for_each(|c| {
            let strategy: Box<dyn StrategyModule> = match &c {
                StrategyConfig::Crossover(c) => Box::new(CrossoverStrategy::from_config(c)),
            };
            strategies.push(strategy);
        });

        strategies
    }
}
