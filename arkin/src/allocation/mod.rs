use std::fmt::Debug;

mod equal;
mod factory;
mod manager;

pub use manager::AllocationManager;

use crate::{
    models::{AllocationEvent, Signal},
    strategies::StrategyId,
};

pub trait Allocation: Debug + Send + Sync {
    fn strategies(&self) -> &[StrategyId];
    fn calculate(&self, signals: &[Signal]) -> Vec<AllocationEvent>;
}
