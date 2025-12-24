use arkin_core::{InstrumentType, VenueName};
use serde::Deserialize;

use crate::features::{DualRangeAlgo, LagAlgo, NormalizeFeatureType, RangeAlgo, RangeData, TwoValueAlgo};
use crate::FillStrategy;

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
    pub name: String,
    pub description: String,
    /// Reference currency for synthetic instruments (e.g., "USD")
    /// Used as quote asset for synthetics: syn-btc-usd, index-global-usd
    pub reference_currency: String,
    pub warmup_steps: u16,
    pub state_ttl: u64,
    pub min_interval: u64,
    pub parallel: bool,
    /// Global filter applied to all features (merged with individual feature filters)
    #[serde(default)]
    pub global_instrument_selector: InstrumentSelector,
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

    #[serde(rename = "liquidity")]
    Liquidity(LiquidityConfig),
}

/// Simple filter for selecting instruments
/// Empty lists/None means "no filter" (include all)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct InstrumentSelector {
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

    /// Filter by instrument symbols (exact match, e.g., ["BTC_SYNTHETIC", "ETH_SYNTHETIC"])
    #[serde(default)]
    pub symbols: Vec<String>,

    /// Filter by synthetic flag: None = all, Some(true) = only synthetic, Some(false) = only real
    #[serde(default)]
    pub synthetic: Option<bool>,
}

/// Dimensions to group by when creating grouped synthetics
/// Specifies which dimensions to aggregate together
#[derive(Debug, Deserialize, Clone, Default)]
pub struct GroupBy {
    /// If true
    #[serde(default)]
    pub base_asset: bool,

    /// Quote assets to aggregate (e.g., ["USDT", "USDC"] means combine BTC-USDT + BTC-USDC → syn-btc-usd)
    #[serde(default)]
    pub quote_asset: Vec<String>,

    /// If true, group by instrument type (creates syn-btc-perpetual-usd, syn-btc-spot-usd)
    #[serde(default)]
    pub instrument_type: bool,

    /// If true, group by venue (creates separate synthetics for each venue)
    #[serde(default)]
    pub venue: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LagConfig {
    pub instrument_selector: InstrumentSelector,
    pub group_by: GroupBy,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub lag: Vec<usize>,
    pub method: Vec<LagAlgo>,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeConfig {
    pub instrument_selector: InstrumentSelector,
    pub group_by: GroupBy,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: Vec<RangeAlgo>,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DualRangeConfig {
    pub instrument_selector: InstrumentSelector,
    pub group_by: GroupBy,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: Vec<DualRangeAlgo>,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwoValueConfig {
    pub instrument_selector_1: InstrumentSelector,
    pub instrument_selector_2: InstrumentSelector,
    pub group_by: GroupBy,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub method: Vec<TwoValueAlgo>,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NormalizeConfig {
    pub instrument_selector: InstrumentSelector,
    pub group_by: GroupBy,
    pub input: Vec<String>,
    pub output: String,
    pub data_location: String,
    pub version: String,
    pub method: NormalizeFeatureType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LiquidityConfig {
    pub instrument_selector: InstrumentSelector,
    pub group_by: GroupBy,
    pub outputs: Vec<String>, // e.g., ["total_volume", "spread", "depth"]
    pub window_seconds: usize, // Rolling window for calculations
    pub fill_strategy: FillStrategy,
}
