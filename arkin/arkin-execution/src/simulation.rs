use std::time::Duration;

use arkin_common::prelude::*;
use rust_decimal::prelude::*;

use crate::{config::SimulationConfig, ExecutionEndpoint};

pub struct SimulationEndpoint {
    _latency: Duration,
    _commission_maker: Decimal,
    _commission_taker: Decimal,
    _max_orders_per_minute: u64,
}

impl SimulationEndpoint {
    pub fn from_config(config: &SimulationConfig) -> Self {
        SimulationEndpoint {
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
