use super::{crossover::CrossoverStrategy, Strategy};
use crate::config::StrategyConfig;

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(configs: &[StrategyConfig]) -> Vec<Box<dyn Strategy>> {
        let mut strategies = Vec::new();

        configs.iter().for_each(|c| {
            let strategy: Box<dyn Strategy> = match &c {
                StrategyConfig::Crossover(c) => Box::new(CrossoverStrategy::from_config(c)),
            };
            strategies.push(strategy);
        });

        strategies
    }
}
