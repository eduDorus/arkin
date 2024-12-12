use arkin_core::FeatureId;
use rust_decimal::Decimal;
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
    pub min_trade_value: Decimal,
    pub allocation_feature_id: FeatureId,
}
