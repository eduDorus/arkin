use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionConfig {
    #[serde(rename = "binance")]
    Binance(BinanceExecutionConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceExecutionConfig {
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}
