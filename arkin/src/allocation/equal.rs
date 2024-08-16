use super::AllocationModule;
use crate::{
    config::EqualConfig,
    models::{Allocation, PositionSnapshot, SignalSnapshot, StrategyId},
};
use rust_decimal::prelude::*;

pub struct EqualAllocation {
    _max_allocation: Decimal,
    _max_allocation_per_instrument: Decimal,
    strategies: Vec<StrategyId>,
}

impl EqualAllocation {
    pub fn from_config(config: &EqualConfig) -> Self {
        EqualAllocation {
            _max_allocation: config.max_allocation,
            _max_allocation_per_instrument: config.max_allocation_per_instrument,
            strategies: config.strategies.clone(),
        }
    }
}

impl AllocationModule for EqualAllocation {
    fn strategies(&self) -> &[StrategyId] {
        &self.strategies
    }

    fn calculate(&self, _signals: &SignalSnapshot, _positions: &PositionSnapshot) -> Vec<Allocation> {
        todo!()
    }
}
