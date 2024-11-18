use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllocationOptimConfig {
    pub allocation_optim: AllocationTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationTypeConfig {
    #[serde(rename = "limited")]
    Limited(LimitedConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitedConfig {
    pub max_allocation: Decimal,
    pub max_allocation_per_signal: Decimal,
}
