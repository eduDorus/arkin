use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionConfig {
    pub execution_manager: ExecutionManagerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionManagerConfig {
    pub default_endpoint: String,
    pub rebalance_threshold: Decimal,
    pub endpoints: Vec<ExecutionEndpointConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionEndpointConfig {
    #[serde(rename = "simulation")]
    Simulation(SimulationConfig),
    #[serde(rename = "binance")]
    Binance(BinanceExecutionConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationConfig {
    pub latency: u64,
    pub commission_maker: Decimal,
    pub commission_taker: Decimal,
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceExecutionConfig {
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}
