use super::Allocation;
use crate::{
    config::EqualConfig,
    models::{AllocationEvent, Signal, Weight},
    strategies::StrategyId,
};
use rust_decimal::prelude::*;

#[derive(Debug)]
pub struct EqualAllocation {
    capital: Decimal,
    max_allocation: Decimal,
    max_allocation_per_instrument: Decimal,
    strategies: Vec<StrategyId>,
}

impl EqualAllocation {
    pub fn from_config(config: &EqualConfig) -> Self {
        EqualAllocation {
            capital: config.capital,
            max_allocation: config.max_allocation,
            max_allocation_per_instrument: config.max_allocation_per_instrument,
            strategies: config.strategies.clone(),
        }
    }
}

impl Allocation for EqualAllocation {
    fn strategies(&self) -> Vec<StrategyId> {
        self.strategies.clone()
    }

    fn calculate(&self, signals: Vec<Signal>) -> Vec<AllocationEvent> {
        let action_signals = signals.iter().filter(|s| s.signal != Weight::from(0.)).count();

        let allocation_per_instrument = self.max_allocation
            / (Decimal::from_usize(action_signals)
                .expect("Failed to convert usize to Decimal")
                .max(Decimal::ONE));

        let allocation = allocation_per_instrument.min(self.max_allocation_per_instrument);
        let allocation_notional = self.capital * allocation;

        signals
            .into_iter()
            .map(|s| {
                AllocationEvent::new(
                    s.event_time,
                    s.instrument,
                    s.strategy_id,
                    (s.signal.value() * allocation_notional).into(),
                )
            })
            .collect()
    }
}
