use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use arkin_core::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsConfig {
    pub insights_service: InsightsServiceConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsServiceConfig {
    pub pipeline: PipelineConfig,
    pub state_lookback: u64,
    pub frequency_secs: u64,
    pub scale_periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "ohlcv")]
    OHLCV(OHLCVConfig),
    #[serde(rename = "time")]
    Time(TimeConfig),

    // Mathematical
    #[serde(rename = "log_return")]
    LogReturn(LogReturnConfig),
    #[serde(rename = "std_dev")]
    StdDev(StdDevConfig),
    #[serde(rename = "sum")]
    Sum(SumConfig),
    #[serde(rename = "signal")]
    SignalStrength(SignalStrengthConfig),

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

    // Forecasting
    #[serde(rename = "catboost")]
    CatBoost(CatBoostConfig),

    // Portfolio Optimization
    #[serde(rename = "mean_variance")]
    MeanVariance(MeanVarianceConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeConfig {
    pub input: FeatureId,
    pub output_day_of_week: FeatureId,
    pub output_hour_of_day: FeatureId,
    pub output_minute_of_day: FeatureId,
    pub output_minute_of_hour: FeatureId,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPConfig {
    pub input_price: FeatureId,
    pub input_quantity: FeatureId,
    pub output: FeatureId,
    pub window: u64,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeCountConfig {
    pub input_side: FeatureId,
    pub output_buy: FeatureId,
    pub output_sell: FeatureId,
    pub output_total: FeatureId,
    pub output_ratio: FeatureId,
    pub window: u64,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogReturnConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdDevConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SumConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MovingAverageConfig {
    pub ma_type: String,
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignalStrengthConfig {
    pub input_first: FeatureId,
    pub input_second: FeatureId,
    pub output: FeatureId,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MACDConfig {
    pub input_fast: FeatureId,
    pub input_slow: FeatureId,
    pub output_signal: FeatureId,
    pub output_histogram: FeatureId,
    pub signal_periods: usize,
    pub smoothing: Decimal,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BollingerBandsConfig {
    pub input_price: FeatureId,
    pub input_sma: FeatureId,
    pub input_stddev: FeatureId,
    pub output_upper: FeatureId,
    pub output_lower: FeatureId,
    pub output_oscillator: FeatureId,
    pub output_width: FeatureId,
    pub sigma: Decimal,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelativeStrengthIndexConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AverageDirectionalIndexConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChaikinMoneyFlowConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChaikinOscillatorConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods_fast: usize,
    pub periods_slow: usize,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeanVarianceConfig {
    pub input_expected_returns: FeatureId,
    pub input_returns: FeatureId,
    pub output: FeatureId,
    pub periods_returns: usize,
    pub risk_aversion: f64, // Lambda: larger values mean more risk
    pub risk_free_rate: f64,
    pub max_exposure_long: f64,
    pub max_exposure_long_per_asset: f64,
    pub max_exposure_short: f64,
    pub max_exposure_short_per_asset: f64,
    pub transaction_cost: f64,
    #[serde(default)]
    pub persist: bool,
}
