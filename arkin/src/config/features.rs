use serde::{Deserialize, Serialize};

use crate::features::FeatureId;

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
    // #[serde(rename = "ema")]
    // EMA(EMAFeatureConfig),
    #[serde(rename = "spread")]
    Spread(SpreadFeatureConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeFeatureConfig {
    pub id: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPFeatureConfig {
    pub id: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAFeatureConfig {
    pub id: FeatureId,
    pub source: FeatureId,
    pub period: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAFeatureConfig {
    pub id: FeatureId,
    pub source: FeatureId,
    pub period: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadFeatureConfig {
    pub id: FeatureId,
    pub front_component: FeatureId,
    pub back_component: FeatureId,
}
