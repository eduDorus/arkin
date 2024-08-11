mod binance;
mod factory;
mod manager;
mod simulation;

pub use factory::ExecutionEndpointFactory;
pub use manager::ExecutionManager;
pub use simulation::SimulationEndpoint;

use crate::models::{ExecutionOrder, Fill, Venue};

pub trait Execution: Send + Sync {
    fn add_orders(&self, orders: Vec<ExecutionOrder>);
}

pub trait ExecutionEndpoint: Send + Sync {
    fn venue(&self) -> &Venue;
    fn place_orders(&self, order: Vec<ExecutionOrder>) -> Vec<Fill>;
}
