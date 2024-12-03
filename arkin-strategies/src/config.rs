use arkin_core::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyConfig {
    pub strategies: Vec<StrategyAlgorithmConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyAlgorithmConfig {
    #[serde(rename = "crossover")]
    Crossover(CrossoverConfig),
    #[serde(rename = "spreader")]
    Spreader(SpreaderConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossoverConfig {
    pub id: Uuid,
    pub fast_ma: FeatureId,
    pub slow_ma: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub id: Uuid,
    pub front_leg: FeatureId,
    pub back_leg: FeatureId,
    pub min_spread: Decimal,
}
