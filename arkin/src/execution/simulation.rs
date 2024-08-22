use std::{sync::Arc, time::Duration};

use crate::{
    config::SimulationConfig,
    models::{ExecutionOrder, Venue},
    state::StateManager,
};
use rust_decimal::prelude::*;

use super::ExecutionEndpoint;

pub struct SimulationEndpoint {
    _state: Arc<StateManager>,
    _latency: Duration,
    _commission_maker: Decimal,
    _commission_taker: Decimal,
    _max_orders_per_minute: u64,
}

impl SimulationEndpoint {
    pub fn from_config(state: Arc<StateManager>, config: &SimulationConfig) -> Self {
        SimulationEndpoint {
            _state: state,
            _latency: Duration::from_millis(config.latency),
            _commission_maker: config.commission_maker,
            _commission_taker: config.commission_taker,
            _max_orders_per_minute: config.max_orders_per_minute,
        }
    }
}

impl ExecutionEndpoint for SimulationEndpoint {
    fn venue(&self) -> &Venue {
        &Venue::Simulation
    }

    fn place_orders(&self, _orders: Vec<ExecutionOrder>) -> Vec<Fill> {
        todo!()
    }
}
