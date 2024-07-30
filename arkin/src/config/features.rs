use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use crate::features::FeatureID;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub frequency: u64,
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "volume")]
    Volume(VolumeFeatureConfig),
    #[serde(rename = "vwap")]
    VWAP(VWAPFeatureConfig),
    #[serde(rename = "sma")]
    SMA(SMAFeatureConfig),
    #[serde(rename = "ema")]
    EMA(EMAFeatureConfig),
    #[serde(rename = "spread")]
    Spread(SpreadFeatureConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeFeatureConfig {
    pub id: FeatureID,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPFeatureConfig {
    pub id: FeatureID,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAFeatureConfig {
    pub id: FeatureID,
    pub source: FeatureID,
    pub period: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAFeatureConfig {
    pub id: FeatureID,
    pub source: FeatureID,
    pub period: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadFeatureConfig {
    pub id: FeatureID,
    pub front_component: FeatureID,
    pub back_component: FeatureID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyConfig {
    #[serde(rename = "crossover")]
    Crossover(CrossoverConfig),
    #[serde(rename = "spreader")]
    Spreader(SpreaderConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossoverConfig {
    pub id: String,
    pub fast: String,
    pub slow: String,
    pub min_spread: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub id: String,
    pub front_leg: String,
    pub back_leg: String,
    pub min_spread: Decimal,
}
