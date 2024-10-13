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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "ohlcv")]
    OHLCV(OHLCVConfig),
    #[serde(rename = "vwap")]
    VWAP(VWAPConfig),
    #[serde(rename = "pct_change")]
    PctChange(PctChangeConfig),
    #[serde(rename = "trade_count")]
    TradeCount(TradeCountConfig),
    #[serde(rename = "std_dev")]
    StdDev(StdDevConfig),
    #[serde(rename = "hist_vol")]
    HistVol(HistVolConfig),
    #[serde(rename = "sma")]
    SMA(SMAConfig),
    #[serde(rename = "ema")]
    EMA(EMAConfig),
    #[serde(rename = "macd")]
    MACD(MACDConfig),
    #[serde(rename = "bollinger_bands")]
    BollingerBands(BollingerBandsConfig),
    #[serde(rename = "rsi")]
    RSI(RSIConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OHLCVConfig {
    pub input_price: NodeId,
    pub input_quantity: NodeId,
    pub output_open: NodeId,
    pub output_high: NodeId,
    pub output_low: NodeId,
    pub output_close: NodeId,
    pub output_volume: NodeId,
    pub output_notional_volume: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPConfig {
    pub input_price: NodeId,
    pub input_quantity: NodeId,
    pub output: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeCountConfig {
    pub input_side: NodeId,
    pub output_buy: NodeId,
    pub output_sell: NodeId,
    pub output_total: NodeId,
    pub output_ratio: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PctChangeConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdDevConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistVolConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub trading_days_per_year: Decimal,
    pub timeframe_in_secs: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub periods: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub periods: usize,
    pub smoothing: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MACDConfig {
    pub input_fast: NodeId,
    pub input_slow: NodeId,
    pub output_signal: NodeId,
    pub output_histogram: NodeId,
    pub signal_periods: usize,
    pub smoothing: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BollingerBandsConfig {
    pub input_price: NodeId,
    pub input_sma: NodeId,
    pub input_stddev: NodeId,
    pub output_upper: NodeId,
    pub output_lower: NodeId,
    pub output_oscillator: NodeId,
    pub output_width: NodeId,
    pub sigma: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RSIConfig {
    pub input_return: NodeId,
    pub output: NodeId,
    pub periods: usize,
}
