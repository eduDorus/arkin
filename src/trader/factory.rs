use std::{collections::HashMap, sync::Arc};

use crate::{config::TraderConfig, state::State};

use super::{allocation::AllocationFactory, strategies::StrategyFactory, DefaultTrader, TraderType};

pub struct TraderFactory {}

impl TraderFactory {
    pub fn create_traders(state: Arc<State>, config: &HashMap<String, TraderConfig>) -> Vec<TraderType> {
        let mut traders = Vec::new();

        for config in config.values() {
            traders.push(Self::create_trader(state.clone(), config));
        }
        traders
    }

    pub fn create_trader(state: Arc<State>, config: &TraderConfig) -> TraderType {
        let strategy = StrategyFactory::from_config(state.clone(), &config.strategy);
        let allocation = AllocationFactory::from_config(state.clone(), &config.allocation);

        let trader = DefaultTrader::builder()
            .with_strategy(strategy)
            .with_allocation(allocation)
            .build()
            .expect("Failed to create trader");

        TraderType::Default(trader)
    }
}
