use std::{sync::Arc, time::Duration};

use crate::{
    config::SimulationConfig,
    constants::TRADE_PRICE_ID,
    features::{FeatureEvent, QueryType},
    models::{AllocationEvent, Price},
    state::State,
};
use rust_decimal::prelude::*;
use tracing::{debug, warn};

pub struct SimulationExecution {
    state: Arc<State>,
    latency: Duration,
    _commission_maker: Decimal,
    _commission_taker: Decimal,
    _max_orders_per_minute: u64,
    _max_order_size_notional: Decimal,
    _min_order_size_notional: Decimal,
}

impl SimulationExecution {
    pub fn from_config(state: Arc<State>, config: &SimulationConfig) -> Self {
        SimulationExecution {
            state,
            latency: Duration::from_millis(config.latency),
            _commission_maker: config.commission_maker,
            _commission_taker: config.commission_taker,
            _max_orders_per_minute: config.max_orders_per_minute,
            _max_order_size_notional: config.max_order_size_notional,
            _min_order_size_notional: config.min_order_size_notional,
        }
    }
}

impl SimulationExecution {
    pub fn allocate(&self, allocations: &[AllocationEvent]) {
        allocations.iter().for_each(|a| {
            let execution_time = a.event_time + self.latency;
            let res =
                self.state
                    .read_features(&a.instrument, &[TRADE_PRICE_ID.clone()], &execution_time, &QueryType::Latest);

            if let Some(price) = res.get(&TRADE_PRICE_ID) {
                if let Some(price) = price.last() {
                    let quantity = a.notional / Price::from(*price);
                    debug!(
                        "Allocating {} {} at price: {} quantity: {} notional: {}",
                        a.instrument, a.notional, price, quantity, a.notional
                    );
                    self.state.add_feature(FeatureEvent::new(
                        "fill_price".into(),
                        a.instrument.to_owned(),
                        execution_time,
                        *price,
                    ));
                    self.state.add_feature(FeatureEvent::new(
                        "fill_quantity".into(),
                        a.instrument.to_owned(),
                        execution_time,
                        quantity.into(),
                    ));
                } else {
                    warn!("No price data found for {}", a.instrument);
                }
            };
        });
    }
}
