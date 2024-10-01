use arkin_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsConfig {
    pub insights_service: InsightsServiceConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsServiceConfig {
    pub pipeline: PipelineConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatestInputConfig {
    pub from: NodeId,
    #[serde(rename = "feature")]
    pub feature_id: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowInputConfig {
    pub from: NodeId,
    #[serde(rename = "feature")]
    pub feature_id: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodInputConfig {
    pub from: NodeId,
    #[serde(rename = "feature")]
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
    #[serde(rename = "cum_sum")]
    CumSum(CumSumFeatureConfig),
    #[serde(rename = "pct_change")]
    PctChange(PctChangeFeatureConfig),
    #[serde(rename = "std_dev")]
    StdDev(StdDevFeatureConfig),
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
pub struct CumSumFeatureConfig {
    pub id: NodeId,
    pub input: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PctChangeFeatureConfig {
    pub id: NodeId,
    pub input: WindowInputConfig,
    pub output: FeatureId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdDevFeatureConfig {
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
