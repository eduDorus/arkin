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
    /// Reference currency for synthetic instruments (e.g., "USD")
    /// Used as quote asset for synthetics: syn-btc-usd, index-global-usd
    pub reference_currency: String,
    pub warmup_steps: u16,
    pub state_ttl: u64,
    pub min_interval: u64,
    pub parallel: bool,
    /// Global filter applied to all features (merged with individual feature filters)
    #[serde(default)]
    pub instrument_filter: InstrumentFilter,
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

/// Aggregation type defines how features are calculated across instruments
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AggregationType {
    /// Calculate per instrument (default behavior)
    /// Each real instrument gets its own feature value
    #[serde(rename = "instrument")]
    Instrument {
        /// Optional filter to select which instruments to calculate this feature for
        #[serde(default)]
        filter: InstrumentFilter,
    },

    /// Create grouped synthetic instruments and calculate features for them
    /// Generates one synthetic per unique group (e.g., syn-btc-usd, syn-eth-usd)
    #[serde(rename = "grouped")]
    Grouped {
        /// Which instruments to include in grouping
        filter: InstrumentFilter,
        /// Dimensions to group by (creates one synthetic per unique combination)
        group_by: GroupBy,
    },

    /// Create a single index synthetic instrument
    /// Aggregates all matching instruments into one (e.g., index-binance-usd, index-global-usd)
    #[serde(rename = "index")]
    Index {
        /// Which instruments to include in the index (can include synthetics)
        filter: InstrumentFilter,
    },
}

impl Default for AggregationType {
    fn default() -> Self {
        Self::Instrument {
            filter: InstrumentFilter::default(),
        }
    }
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

    /// Filter by instrument symbols (exact match, e.g., ["BTC_SYNTHETIC", "ETH_SYNTHETIC"])
    #[serde(default)]
    pub symbols: Vec<String>,

    /// Filter by synthetic flag: None = all, Some(true) = only synthetic, Some(false) = only real
    #[serde(default)]
    pub synthetic: Option<bool>,
}

impl InstrumentFilter {
    /// Merge this filter with a global filter
    /// Feature-level filter takes precedence (if not empty), otherwise uses global filter
    pub fn merge_with_global(&self, global: &InstrumentFilter) -> InstrumentFilter {
        InstrumentFilter {
            base_asset: if !self.base_asset.is_empty() {
                self.base_asset.clone()
            } else {
                global.base_asset.clone()
            },
            quote_asset: if !self.quote_asset.is_empty() {
                self.quote_asset.clone()
            } else {
                global.quote_asset.clone()
            },
            instrument_type: if !self.instrument_type.is_empty() {
                self.instrument_type.clone()
            } else {
                global.instrument_type.clone()
            },
            venue: if !self.venue.is_empty() {
                self.venue.clone()
            } else {
                global.venue.clone()
            },
            symbols: if !self.symbols.is_empty() {
                self.symbols.clone()
            } else {
                global.symbols.clone()
            },
            synthetic: self.synthetic.or(global.synthetic),
        }
    }
}

/// Dimensions to group by when creating grouped synthetics
/// Specifies which dimensions to aggregate together
#[derive(Debug, Deserialize, Clone, Default)]
pub struct GroupBy {
    /// Quote assets to aggregate (e.g., ["USDT", "USDC"] means combine BTC-USDT + BTC-USDC → syn-btc-usd)
    #[serde(default)]
    pub quote_asset: Vec<String>,

    /// If true, group by instrument type (creates syn-btc-perpetual-usd, syn-btc-spot-usd)
    #[serde(default)]
    pub instrument_type: bool,

    /// Optional venue to restrict grouping (e.g., Some(BinanceUsdmFutures) means only group within that venue)
    #[serde(default)]
    pub venue: Option<VenueName>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LagConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub lag: Vec<usize>,
    pub method: LagAlgo,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: RangeAlgo,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DualRangeConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub data: Vec<RangeData>,
    pub method: DualRangeAlgo,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwoValueConfig {
    #[serde(default)]
    pub aggregation_type: AggregationType,
    #[serde(default)]
    pub input_filter_1: Option<InstrumentFilter>,
    #[serde(default)]
    pub input_filter_2: Option<InstrumentFilter>,
    pub input_1: Vec<String>,
    pub input_2: Vec<String>,
    pub output: Vec<String>,
    pub method: TwoValueAlgo,
    pub fill_strategy: FillStrategy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NormalizeConfig {
    pub aggregation_type: AggregationType,
    pub input: Vec<String>,
    pub output: String,
    pub data_location: String,
    pub version: String,
    pub method: NormalizeFeatureType,
}
