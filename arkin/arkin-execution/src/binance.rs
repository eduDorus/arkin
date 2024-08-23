use arkin_common::prelude::*;
use rust_decimal::Decimal;

use crate::{config::BinanceExecutionConfig, ExecutionEndpoint};

#[derive(Clone)]
#[allow(unused)]
pub struct BinanceEndpoint {
    max_orders_per_minute: u64,
    max_order_size_notional: Decimal,
    min_order_size_notional: Decimal,
}

impl BinanceEndpoint {
    pub fn from_config(config: &BinanceExecutionConfig) -> Self {
        BinanceEndpoint {
            max_orders_per_minute: config.max_orders_per_minute,
            max_order_size_notional: config.max_order_size_notional,
            min_order_size_notional: config.min_order_size_notional,
        }
    }
}

impl ExecutionEndpoint for BinanceEndpoint {
    fn venue(&self) -> &Venue {
        &Venue::Binance
    }

    fn place_orders(&self, _order: Vec<ExecutionOrder>) -> Vec<Fill> {
        todo!()
    }
}
