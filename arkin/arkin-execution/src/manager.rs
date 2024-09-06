use std::collections::HashMap;

use arkin_common::prelude::*;

use crate::{config::ExecutionManagerConfig, factory::ExecutionEndpointFactory};

pub trait Executor: Send + Sync {
    fn add_orders(&self, orders: Vec<ExecutionOrder>);
}

pub trait ExecutionEndpoint: Send + Sync {
    fn venue(&self) -> &Venue;
    fn place_orders(&self, order: Vec<ExecutionOrder>) -> Vec<Fill>;
}

pub struct ExecutionManager {
    _endpoints: HashMap<Venue, Box<dyn ExecutionEndpoint>>,
    _default_endpoint: Venue,
}

impl ExecutionManager {
    pub fn from_config(config: &ExecutionManagerConfig) -> Self {
        let endpoints = ExecutionEndpointFactory::from_config(&config.endpoints)
            .into_iter()
            .map(|endpoint| (endpoint.venue().clone(), endpoint))
            .collect();
        Self {
            _endpoints: endpoints,
            _default_endpoint: config.default_endpoint.parse().expect("Invalid venue"),
        }
    }

    pub fn process(&self, _allocations: &AllocationSnapshot) {}
}
