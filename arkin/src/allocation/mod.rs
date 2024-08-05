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
    fn strategies(&self) -> Vec<StrategyId>;
    fn calculate(&self, signals: Vec<Signal>) -> Vec<AllocationEvent>;
}
