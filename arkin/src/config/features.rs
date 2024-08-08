use serde::{Deserialize, Serialize};

use crate::features::{FeatureId, Latest, NodeId, Period, Window};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub frequency: u64,
    pub features: Vec<FeatureConfig>,
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
    #[serde(rename = "position")]
    Position(PositionConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountFeatureConfig {
    pub id: NodeId,
    pub input: Window,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SumFeatureConfig {
    pub id: NodeId,
    pub input: Window,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeanFeatureConfig {
    pub id: NodeId,
    pub input: Window,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPFeatureConfig {
    pub id: NodeId,
    pub input_price: Window,
    pub input_quantity: Window,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAFeatureConfig {
    pub id: NodeId,
    pub input: Period,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadFeatureConfig {
    pub id: NodeId,
    pub input_front: Latest,
    pub input_back: Latest,
    pub output: FeatureId,
    pub absolute: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionConfig {
    pub id: NodeId,
    pub input_position_price: Latest,
    pub input_position_quantity: Latest,
    pub input_fill_price: Window,
    pub input_fill_quantity: Window,
    pub output_price: FeatureId,
    pub output_quantity: FeatureId,
}
