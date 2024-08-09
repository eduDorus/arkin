use serde::{Deserialize, Serialize};

use crate::features::{FeatureId, NodeId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub frequency: u64,
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatestInputConfig {
    pub from: NodeId,
    pub feature_id: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowInputConfig {
    pub from: NodeId,
    pub feature_id: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodInputConfig {
    pub from: NodeId,
    pub feature_id: FeatureId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "count")]
    Count(CountFeatureConfig),
    #[serde(rename = "sum")]
    Sum(SumFeatureConfig),
    #[serde(rename = "mean")]
    Mean(MeanFeatureConfig),
    #[serde(rename = "vwap")]
    VWAP(VWAPFeatureConfig),
    #[serde(rename = "sma")]
    SMA(SMAFeatureConfig),
    #[serde(rename = "spread")]
    Spread(SpreadFeatureConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountFeatureConfig {
    pub id: NodeId,
    pub input: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SumFeatureConfig {
    pub id: NodeId,
    pub input: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeanFeatureConfig {
    pub id: NodeId,
    pub input: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPFeatureConfig {
    pub id: NodeId,
    pub input_price: WindowInputConfig,
    pub input_quantity: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAFeatureConfig {
    pub id: NodeId,
    pub input: PeriodInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadFeatureConfig {
    pub id: NodeId,
    pub input_front: LatestInputConfig,
    pub input_back: LatestInputConfig,
    pub output: FeatureId,
    pub absolute: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionConfig {
    pub id: NodeId,
    pub input_position_price: LatestInputConfig,
    pub input_position_quantity: LatestInputConfig,
    pub input_fill_price: WindowInputConfig,
    pub input_fill_quantity: WindowInputConfig,
    pub output_price: FeatureId,
    pub output_quantity: FeatureId,
}
