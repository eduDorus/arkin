use std::{sync::Arc, time::Duration};

use crate::{
    config::SimulationConfig,
    models::{Fill, Order, Tick, Venue},
    state::StateManager,
};
use rust_decimal::prelude::*;
use tracing::{debug, info, warn};

use super::ExecutionEndpoint;

pub struct SimulationEndpoint {
    state: Arc<StateManager>,
    latency: Duration,
    _commission_maker: Decimal,
    commission_taker: Decimal,
    _max_orders_per_minute: u64,
}

impl SimulationEndpoint {
    pub fn from_config(state: Arc<StateManager>, config: &SimulationConfig) -> Self {
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
                if let Some(tick) = self
                    .state
                    .latest_event_by_instrument::<Tick>(&o.instrument, &(o.event_time + self.latency))
                {
                    debug!("Placing order: {}", o);
                    Some((o, tick.mid_price()))
                } else {
                    warn!("Order rejected: {}", o);
                    None
                }
            })
            .map(|(o, p)| {
                Fill::new(
                    o.event_time,
                    o.instrument,
                    o.order_id,
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
