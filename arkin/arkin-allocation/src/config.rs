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
    #[serde(rename = "equal")]
    Equal(EqualConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EqualConfig {
    pub max_allocation: Decimal,
    pub max_allocation_per_underlier: Decimal,
}
