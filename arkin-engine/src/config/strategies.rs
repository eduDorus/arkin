use arkin_core::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "forecast")]
    Forecast(ForecastConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossoverConfig {
    pub name: String,
    pub fast_ma: FeatureId,
    pub slow_ma: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub name: String,
    pub front_leg: FeatureId,
    pub back_leg: FeatureId,
    pub min_spread: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastConfig {
    pub name: String,
    pub inputs: Vec<FeatureId>,
    pub threshold: f64,
}
