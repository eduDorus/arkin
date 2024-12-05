use arkin_core::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsConfig {
    pub insights_service: InsightsServiceConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsightsServiceConfig {
    pub pipeline: PipelineConfig,
    pub state_lookback: u64,
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
    // #[serde(rename = "vwap")]
    // VWAP(VWAPConfig),
    // #[serde(rename = "pct_change")]
    // PctChange(PctChangeConfig),
    // #[serde(rename = "trade_count")]
    // TradeCount(TradeCountConfig),
    // #[serde(rename = "std_dev")]
    // StdDev(StdDevConfig),
    // #[serde(rename = "hist_vol")]
    // HistVol(HistVolConfig),
    #[serde(rename = "sma")]
    SMA(SMAConfig),
    // #[serde(rename = "ema")]
    // EMA(EMAConfig),
    // #[serde(rename = "macd")]
    // MACD(MACDConfig),
    // #[serde(rename = "bb")]
    // BB(BollingerBandsConfig),
    // #[serde(rename = "rsi")]
    // RSI(RelativeStrengthIndexConfig),
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPConfig {
    pub input_price: FeatureId,
    pub input_quantity: FeatureId,
    pub output: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeCountConfig {
    pub input_side: FeatureId,
    pub output_buy: FeatureId,
    pub output_sell: FeatureId,
    pub output_total: FeatureId,
    pub output_ratio: FeatureId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PctChangeConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdDevConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistVolConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub trading_days_per_year: Decimal,
    pub timeframe_in_secs: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
    pub smoothing: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MACDConfig {
    pub input_fast: FeatureId,
    pub input_slow: FeatureId,
    pub output_signal: FeatureId,
    pub output_histogram: FeatureId,
    pub signal_periods: usize,
    pub smoothing: Decimal,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelativeStrengthIndexConfig {
    pub input_return: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyFlowIndexConfig {
    pub input_price: FeatureId,
    pub input_quantity: FeatureId,
    pub output: FeatureId,
    pub periods: usize,
}
