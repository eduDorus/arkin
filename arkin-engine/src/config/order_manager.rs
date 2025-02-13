use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderManagersConfig {
    pub order_managers: OrderManagerTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderManagerTypeConfig {
    pub single_executor: Option<SingleExecutorTypeConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleExecutorTypeConfig {
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}
