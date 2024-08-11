use super::{Execution, ExecutionEndpoint, ExecutionEndpointFactory};
use crate::{
    config::ExecutionManagerConfig,
    models::{ExecutionOrder, Venue},
    state::StateManager,
};
use std::{collections::HashMap, sync::Arc};

pub struct ExecutionManager {
    _state: Arc<StateManager>,
    _endpoints: HashMap<Venue, Box<dyn ExecutionEndpoint>>,
    _default_endpoint: Venue,
}

impl ExecutionManager {
    pub fn from_config(state: Arc<StateManager>, config: &ExecutionManagerConfig) -> Self {
        let endpoints = ExecutionEndpointFactory::from_config(state.clone(), &config.endpoints)
            .into_iter()
            .map(|endpoint| (endpoint.venue().clone(), endpoint))
            .collect();
        Self {
            _state: state,
            _endpoints: endpoints,
            _default_endpoint: config.default_endpoint.parse().expect("Invalid venue"),
        }
    }
}

impl Execution for ExecutionManager {
    fn add_orders(&self, _orders: Vec<ExecutionOrder>) {
        todo!()
    }
}
