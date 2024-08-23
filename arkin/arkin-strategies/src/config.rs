use arkin_common::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyManagerConfig {
    pub strategies: Vec<StrategyConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyConfig {
    #[serde(rename = "crossover")]
    Crossover(CrossoverConfig),
    // #[serde(rename = "spreader")]
    // Spreader(SpreaderConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossoverConfig {
    pub id: StrategyId,
    pub price_spread_id: FeatureId,
    pub volume_spread_id: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub id: StrategyId,
    pub front_leg: FeatureId,
    pub back_leg: FeatureId,
    pub min_spread: Decimal,
}
