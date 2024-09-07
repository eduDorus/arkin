use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllocationConfig {
    pub allocation_manager: AllocationManagerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllocationManagerConfig {
    pub module: AllocationModuleConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationModuleConfig {
    #[serde(rename = "simple")]
    Simple(SimpleConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleConfig {
    pub max_allocation: Decimal,
    pub max_allocation_per_underlier: Decimal,
}
