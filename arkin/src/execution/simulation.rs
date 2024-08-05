use std::{sync::Arc, time::Duration};

use crate::{
    config::SimulationConfig,
    features::QueryType,
    models::{AllocationEvent, Price},
    state::State,
};
use rust_decimal::prelude::*;
use tracing::{info, warn};

pub struct SimulationExecution {
    state: Arc<State>,
    latency: Duration,
    _max_orders_per_minute: u64,
    _max_order_size_notional: Decimal,
    _min_order_size_notional: Decimal,
}

impl SimulationExecution {
    pub fn from_config(state: Arc<State>, config: &SimulationConfig) -> Self {
        SimulationExecution {
            state,
            latency: Duration::from_millis(config.latency),
            _max_orders_per_minute: config.max_orders_per_minute,
            _max_order_size_notional: config.max_order_size_notional,
            _min_order_size_notional: config.min_order_size_notional,
        }
    }
}

impl SimulationExecution {
    pub fn allocate(&self, allocations: &[AllocationEvent]) {
        allocations.iter().for_each(|a| {
            let res = self.state.query(
                &a.instrument,
                &["trade_price".into()],
                &(a.event_time + self.latency),
                &QueryType::Latest,
            );
            info!("Got data: {:?}", res);

            if let Some(price) = res.get(&"trade_price".into()) {
                if let Some(price) = price.last() {
                    info!(
                        "Allocating {} {} at price {} with quantity {}",
                        a.instrument,
                        a.notional,
                        price,
                        a.notional / Price::from(*price)
                    );
                } else {
                    warn!("No price data found for {}", a.instrument);
                }
            };
        });
    }
}
