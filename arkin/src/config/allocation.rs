use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationConfig {
    #[serde(rename = "limited")]
    Limited(LimitedAllocationConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitedAllocationConfig {
    pub max_allocation: Decimal,
}
