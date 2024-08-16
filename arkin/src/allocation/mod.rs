mod equal;
mod factory;
mod manager;

pub use manager::AllocationManager;

use crate::models::{Allocation, PositionSnapshot, SignalSnapshot, StrategyId};

pub trait AllocationModule: Send + Sync {
    fn strategies(&self) -> &[StrategyId];
    fn calculate(&self, signals: &SignalSnapshot, positions: &PositionSnapshot) -> Vec<Allocation>;
}
