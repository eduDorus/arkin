use std::{sync::Arc, time::Duration};

use crate::{
    config::SimulationConfig,
    models::{Fill, Order, Venue},
    state::State,
};
use rust_decimal::prelude::*;
use tracing::{info, warn};

use super::ExecutionEndpoint;

pub struct SimulationEndpoint {
    state: Arc<State>,
    latency: Duration,
    _commission_maker: Decimal,
    commission_taker: Decimal,
    _max_orders_per_minute: u64,
}

impl SimulationEndpoint {
    pub fn from_config(state: Arc<State>, config: &SimulationConfig) -> Self {
        SimulationEndpoint {
            state,
            latency: Duration::from_millis(config.latency),
            _commission_maker: config.commission_maker,
            commission_taker: config.commission_taker,
            _max_orders_per_minute: config.max_orders_per_minute,
        }
    }
}

impl ExecutionEndpoint for SimulationEndpoint {
    fn venue(&self) -> &Venue {
        &Venue::Simulation
    }

    fn place_orders(&self, orders: Vec<Order>) -> Vec<Fill> {
        // Simulate order placement
        orders
            .into_iter()
            .filter_map(|o| {
                if let Some(price) = self.state.latest_price(&o.instrument, &(o.event_time + self.latency)) {
                    info!("Placing order: {}", o);
                    Some((o, price))
                } else {
                    warn!("Order rejected: {}", o);
                    None
                }
            })
            .map(|(o, p)| {
                Fill::new(
                    o.event_time,
                    o.instrument,
                    Some(o.order_id),
                    o.strategy_id,
                    p,
                    o.quantity,
                    (p * o.quantity) * self.commission_taker,
                )
            })
            .map(|f| {
                info!("Order filled: {}", f);
                f
            })
            .collect()
    }
}
