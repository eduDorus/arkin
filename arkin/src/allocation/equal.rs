use std::sync::Arc;

use super::AllocationModule;
use crate::{
    config::EqualConfig,
    models::{Allocation, Signal, Weight},
    state::StateManager,
    strategies::StrategyId,
};
use rust_decimal::prelude::*;

pub struct EqualAllocation {
    state: Arc<StateManager>,
    max_allocation: Decimal,
    max_allocation_per_instrument: Decimal,
    strategies: Vec<StrategyId>,
}

impl EqualAllocation {
    pub fn from_config(state: Arc<StateManager>, config: &EqualConfig) -> Self {
        EqualAllocation {
            state,
            max_allocation: config.max_allocation,
            max_allocation_per_instrument: config.max_allocation_per_instrument,
            strategies: config.strategies.clone(),
        }
    }
}

impl AllocationModule for EqualAllocation {
    fn strategies(&self) -> &[StrategyId] {
        &self.strategies
    }

    fn calculate(&self, signals: &[Signal]) -> Vec<Allocation> {
        let action_signals = signals.iter().filter(|s| s.signal != Weight::from(0.)).count();

        let allocation_per_instrument = self.max_allocation
            / (Decimal::from_usize(action_signals)
                .expect("Failed to convert usize to Decimal")
                .max(Decimal::ONE));

        let allocation = allocation_per_instrument.min(self.max_allocation_per_instrument);

        signals
            .iter()
            .map(|s| {
                Allocation::new(
                    s.event_time,
                    s.instrument.clone(),
                    s.strategy_id.clone(),
                    (s.signal.value() * allocation_notional).into(),
                )
            })
            .collect()
    }
}
