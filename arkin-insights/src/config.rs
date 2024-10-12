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
    #[serde(rename = "ohlc")]
    OHLC(OHLCConfig),
    #[serde(rename = "pct_change")]
    PctChange(PctChangeConfig),
    #[serde(rename = "trade_count")]
    TradeCount(TradeCountConfig),
    #[serde(rename = "stddev")]
    StdDev(StdDevConfig),
    #[serde(rename = "hist_vol")]
    HistVol(HistVolConfig),
    #[serde(rename = "sma")]
    SMA(SMAConfig),
    #[serde(rename = "ema")]
    EMA(EMAConfig),
    #[serde(rename = "macd")]
    MACD(MACDConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OHLCConfig {
    pub input: NodeId,
    pub open_output: NodeId,
    pub high_output: NodeId,
    pub low_output: NodeId,
    pub close_output: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PctChangeConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistVolConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub trading_days: Decimal,
    pub timeframe_in_secs: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeCountConfig {
    pub input: NodeId,
    pub buy_output: NodeId,
    pub sell_output: NodeId,
    pub total_output: NodeId,
    pub ratio_output: NodeId,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SumFeatureConfig {
    pub inputs: Vec<NodeId>,
    pub outputs: Vec<NodeId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdDevConfig {
    pub input: NodeId,
    pub output: NodeId,
    pub window: u64,
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
    pub fast_input: NodeId,
    pub slow_input: NodeId,
    pub signal_output: NodeId,
    pub histogram_output: NodeId,
    pub signal_periods: usize,
    pub smoothing: Decimal,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum FeatureConfig {
//     #[serde(rename = "count")]
//     Count(CountFeatureConfig),
//     #[serde(rename = "sum")]
//     Sum(SumFeatureConfig),
//     #[serde(rename = "mean")]
//     Mean(MeanFeatureConfig),
//     #[serde(rename = "cum_sum")]
//     CumSum(CumSumFeatureConfig),
//     #[serde(rename = "pct_change")]
//     PctChange(PctChangeFeatureConfig),
//     #[serde(rename = "std_dev")]
//     StdDev(StdDevFeatureConfig),
//     #[serde(rename = "vwap")]
//     VWAP(VWAPFeatureConfig),
//     #[serde(rename = "sma")]
//     SMA(SMAFeatureConfig),
//     #[serde(rename = "spread")]
//     Spread(SpreadFeatureConfig),
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CountFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct SumFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct MeanFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CumSumFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct PctChangeFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct StdDevFeatureConfig {
//     pub id: NodeId,
//     pub input: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct VWAPFeatureConfig {
//     pub id: NodeId,
//     pub input_price: WindowInputConfig,
//     pub input_quantity: WindowInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct SMAFeatureConfig {
//     pub id: NodeId,
//     pub input: PeriodInputConfig,
//     pub output: FeatureId,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct SpreadFeatureConfig {
//     pub id: NodeId,
//     pub input_front: LatestInputConfig,
//     pub input_back: LatestInputConfig,
//     pub output: FeatureId,
//     pub absolute: bool,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct PositionConfig {
//     pub id: NodeId,
//     pub input_position_price: LatestInputConfig,
//     pub input_position_quantity: LatestInputConfig,
//     pub input_fill_price: WindowInputConfig,
//     pub input_fill_quantity: WindowInputConfig,
//     pub output_price: FeatureId,
//     pub output_quantity: FeatureId,
// }
