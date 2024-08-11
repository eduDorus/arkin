mod equal;
mod factory;
mod manager;

pub use manager::AllocationManager;

use crate::{
    models::{ExecutionOrder, Signal},
    strategies::StrategyId,
};

pub trait AllocationModule: Send + Sync {
    fn strategies(&self) -> &[StrategyId];
    fn calculate(&self, signals: &[Signal]) -> Vec<ExecutionOrder>;
}
