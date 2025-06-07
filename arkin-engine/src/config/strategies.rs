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
    // #[serde(rename = "agent")]
    // Agent(AgentConfig),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub model_location: String,
    pub model_name: String,
    pub model_version: String,
    pub action_space: Vec<Decimal>,
    pub n_layers: usize,
    pub hidden_size: usize,
    pub inputs: Vec<FeatureId>,
    pub input_change: FeatureId,
    pub commission_rate: Decimal,
}
