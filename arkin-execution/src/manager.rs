use std::collections::HashMap;

use arkin_core::prelude::*;
use rust_decimal::prelude::*;

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

    pub fn process(&self, _allocations: &AllocationSnapshot) {
        todo!("Implement me")
    }

    pub fn process_backtest(&self, allocations: &AllocationSnapshot, market_snapshot: &MarketSnapshot) -> Vec<Fill> {
        // Fill all orders
        allocations
            .orders
            .iter()
            .map(|o| {
                let tick = market_snapshot.last_tick(&o.instrument).unwrap();
                let price = match &o.side {
                    Side::Buy => tick.ask_price,
                    Side::Sell => tick.bid_price,
                };
                let commission = price * &o.remaining_quantity() * Decimal::from_f64(0.0002).unwrap();
                Fill::new(
                    Account::new(Venue::Backtest, "backtest", 100000),
                    o.instrument.clone(),
                    o.strategy,
                    o.id,
                    o.id,
                    Venue::Backtest,
                    o.side,
                    price,
                    o.remaining_quantity(),
                )
            })
            .collect()
    }
}
