use arkin_core::{InstrumentType, VenueName};
use serde::Deserialize;

use crate::features::{DualRangeAlgo, LagAlgo, NormalizeFeatureType, RangeAlgo, RangeData, TwoValueAlgo};

#[derive(Debug, Deserialize, Clone)]
pub struct InsightsConfig {
    pub insights_service: InsightsServiceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InsightsServiceConfig {
    pub pipeline: PipelineConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PipelineConfig {
    pub version: String,
    pub warmup_steps: u16,
    pub state_ttl: u64,
    pub frequency_secs: u64,
    pub features: Vec<FeatureConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum FeatureConfig {
    // #[serde(rename = "time")]
    // Time(TimeConfig),
    #[serde(rename = "lag")]
    Lag(LagConfig),

    #[serde(rename = "two_value")]
    TwoValue(TwoValueConfig),

    #[serde(rename = "range")]
    Range(RangeConfig),

    #[serde(rename = "dual_range")]
    DualRange(DualRangeConfig),

    // Transformers
    #[serde(rename = "normalize")]
    Normalize(NormalizeConfig),
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct TimeConfig {
//     pub input: String,
//     pub output_day_of_week: String,
//     pub output_hour_of_day: String,
//     pub output_minute_of_day: String,
//     pub output_minute_of_hour: String,
//     #[serde(default)]
//     pub persist: bool,
// }

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    #[default]
    Instrument, // Calculate per instrument (current behavior)
    Index,   // Single aggregate across filtered instruments
    Grouped, // Multiple aggregates, one per group
}

/// Simple filter for selecting instruments
/// Empty lists/None means "no filter" (include all)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct InstrumentFilter {
    /// Filter by base asset symbols (e.g., ["BTC", "ETH"])
    #[serde(default)]
    pub base_asset: Vec<String>,

    /// Filter by quote asset symbols (e.g., ["USDT", "USDC"])
    #[serde(default)]
    pub quote_asset: Vec<String>,

    /// Filter by instrument type (e.g., ["perpetual", "spot"])
    #[serde(default)]
    pub instrument_type: Vec<InstrumentType>,

    /// Filter by venue (e.g., ["binance", "okx"])
    #[serde(default)]
    pub venue: Vec<VenueName>,

    /// Filter by synthetic flag: None = all, Some(true) = only synthetic, Some(false) = only real
    #[serde(default)]
    pub synthetic: Option<bool>,
}

/// Dimensions to group by when aggregation_type = Grouped
/// true means create separate features for each unique value
#[derive(Debug, Deserialize, Clone, Default)]
pub struct GroupBy {
    /// Group by base asset (e.g., one feature per BTC, ETH, etc.)
    #[serde(default)]
    pub base_asset: bool,

    /// Group by quote asset (e.g., one feature per USDT, USDC, etc.)
    #[serde(default)]
    pub quote_asset: bool,

    /// Group by instrument type (e.g., one feature per spot, perpetual, etc.)
    #[serde(default)]
    pub instrument_type: bool,

    /// Group by venue (e.g., one feature per binance, okx, etc.)
    #[serde(default)]
    pub venue: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LagConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    #[serde(default)]
    pub filter: InstrumentFilter,
    #[serde(default)]
    pub group_by: GroupBy,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub lag: Vec<usize>,
    pub method: LagAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    #[serde(default)]
    pub filter: InstrumentFilter,
    #[serde(default)]
    pub group_by: GroupBy,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: RangeAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DualRangeConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    #[serde(default)]
    pub filter: InstrumentFilter,
    #[serde(default)]
    pub group_by: GroupBy,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: DualRangeAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwoValueConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    #[serde(default)]
    pub filter: InstrumentFilter,
    #[serde(default)]
    pub group_by: GroupBy,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub horizons: Vec<RangeData>,
    pub method: TwoValueAlgo,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NormalizeConfig {
    pub aggregation_type: AggregationType,
    pub input: Vec<String>,
    pub output: String,
    pub data_location: String,
    pub version: String,
    pub method: NormalizeFeatureType,
    #[serde(default)]
    pub persist: bool,
}
