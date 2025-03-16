use serde::Deserialize;

use arkin_core::prelude::*;

use crate::features::{DualRangeAlgo, LagAlgo, NormalizeFeatureType, RangeAlgo, RangeData, TwoValueAlgo};

#[derive(Debug, Deserialize, Clone)]
pub struct InsightsConfig {
    pub insights_service: InsightsServiceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InsightsServiceConfig {
    pub pipeline: PipelineConfig,
    pub state_lookback: u64,
    pub frequency_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "ohlcv")]
    OHLCV(OHLCVConfig),
    #[serde(rename = "time")]
    Time(TimeConfig),

    #[serde(rename = "lag")]
    Lag(LagConfig),

    #[serde(rename = "two_value")]
    TwoValue(TwoValueConfig),

    #[serde(rename = "range")]
    Range(RangeConfig),

    #[serde(rename = "dual_range")]
    DualRange(DualRangeConfig),

    // Technical Analysis
    #[serde(rename = "ma")]
    MA(MovingAverageConfig),
    #[serde(rename = "rsi")]
    RSI(RelativeStrengthIndexConfig),
    #[serde(rename = "adx")]
    ADX(AverageDirectionalIndexConfig),
    #[serde(rename = "cmf")]
    CMF(ChaikinMoneyFlowConfig),
    #[serde(rename = "co")]
    CO(ChaikinOscillatorConfig),

    // Transformers
    #[serde(rename = "normalize")]
    Normalize(NormalizeConfig),

    // Forecasting
    #[serde(rename = "catboost")]
    CatBoost(CatBoostConfig),

    #[serde(rename = "onnx")]
    Onnx(OnnxConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct OHLCVConfig {
    pub input_price: FeatureId,
    pub input_quantity: FeatureId,
    pub output_open: FeatureId,
    pub output_high: FeatureId,
    pub output_low: FeatureId,
    pub output_close: FeatureId,
    pub output_typical_price: FeatureId,
    pub output_vwap: FeatureId,
    pub output_volume: FeatureId,
    pub output_buy_volume: FeatureId,
    pub output_sell_volume: FeatureId,
    pub output_notional_volume: FeatureId,
    pub output_buy_notional_volume: FeatureId,
    pub output_sell_notional_volume: FeatureId,
    pub output_trade_count: FeatureId,
    pub output_buy_trade_count: FeatureId,
    pub output_sell_trade_count: FeatureId,
    pub window: u64,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimeConfig {
    pub input: FeatureId,
    pub output_day_of_week: FeatureId,
    pub output_hour_of_day: FeatureId,
    pub output_minute_of_day: FeatureId,
    pub output_minute_of_hour: FeatureId,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LagConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub lag: usize,
    pub method: LagAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub data: RangeData,
    pub method: RangeAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DualRangeConfig {
    pub input_1: FeatureId,
    pub input_2: FeatureId,
    pub output: FeatureId,
    pub data: RangeData,
    pub method: DualRangeAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwoValueConfig {
    pub input_1: FeatureId,
    pub input_2: FeatureId,
    pub output: FeatureId,
    pub method: TwoValueAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MovingAverageConfig {
    pub ma_type: String,
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RelativeStrengthIndexConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AverageDirectionalIndexConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChaikinMoneyFlowConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChaikinOscillatorConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods_fast: usize,
    pub periods_slow: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NormalizeConfig {
    pub input: Vec<FeatureId>,
    pub output: FeatureId,
    pub data_location: String,
    pub method: NormalizeFeatureType,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CatBoostConfig {
    pub model_location: String,
    pub model_name: String,
    pub model_version: String,
    pub input_numerical: Vec<FeatureId>,
    pub input_categorical: Vec<FeatureId>,
    pub output: FeatureId,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OnnxConfig {
    pub model_location: String,
    pub model_name: String,
    pub model_version: String,
    pub input: Vec<FeatureId>,
    pub output: FeatureId,
    pub sequence_length: usize,
    pub target_feature: FeatureId,
    pub quantile_data_location: String,
    #[serde(default)]
    pub persist: bool,
}
