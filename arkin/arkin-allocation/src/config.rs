use arkin_common::prelude::*;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllocationManagerConfig {
    pub allocations: Vec<AllocationConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationConfig {
    #[serde(rename = "equal")]
    Equal(EqualConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EqualConfig {
    pub capital: Decimal,
    pub max_allocation: Decimal,
    pub max_allocation_per_instrument: Decimal,
    pub strategies: Vec<StrategyId>,
}
