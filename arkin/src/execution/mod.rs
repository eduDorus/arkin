mod binance;
mod factory;
mod manager;
mod simulation;

pub use factory::ExecutionEndpointFactory;
pub use manager::ExecutionManager;
pub use simulation::SimulationEndpoint;

use crate::models::{Allocation, Fill, Order, Venue};

pub trait Execution: Send + Sync {
    fn allocate(&self, allocation: &[Allocation]);
}

pub trait ExecutionEndpoint: Send + Sync {
    fn venue(&self) -> &Venue;
    fn place_orders(&self, order: Vec<Order>) -> Vec<Fill>;
}
