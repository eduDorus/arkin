use std::sync::Arc;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use super::{
    Account, Asset, AssetType, Instance, InstanceType, Instrument, InstrumentStatus, InstrumentType, Pipeline,
    Strategy, Venue, VenueName, VenueType,
};

/// Query builder for filtering assets (list operations)
///
/// Empty vectors mean "no filter" (include all).
/// Use this for list_assets operations where you want to
/// specify multiple options for each criteria.
///
/// # Examples
/// ```ignore
/// // Query crypto and stablecoin assets with specific symbols
/// let query = AssetListQuery::builder()
///     .symbols(vec!["BTC", "ETH", "USDT"])
///     .asset_types(vec![AssetType::Crypto, AssetType::Stablecoin])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct AssetListQuery {
    /// Filter by specific asset symbols (case-insensitive)
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub symbols: Vec<String>,

    /// Filter by asset type
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub asset_types: Vec<AssetType>,
}

impl AssetListQuery {
    /// Check if an asset matches this query
    pub fn matches(&self, asset: &Asset) -> bool {
        // Check symbols filter
        if !self.symbols.is_empty() && !self.symbols.iter().any(|s| s.eq_ignore_ascii_case(&asset.symbol)) {
            return false;
        }

        // Check asset types filter
        if !self.asset_types.is_empty() && !self.asset_types.contains(&asset.asset_type) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty() && self.asset_types.is_empty()
    }
}

/// Query for finding a single asset with exact criteria
///
/// Use this for get_asset operations where you want to specify
/// exact matches for symbol, asset type, etc.
///
/// # Examples
/// ```ignore
/// // Find BTC crypto asset
/// let query = AssetQuery::builder()
///     .symbol("BTC")
///     .asset_type(AssetType::Crypto)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct AssetQuery {
    /// Exact asset id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,

    /// Exact asset symbol (case-insensitive)
    #[builder(default, setter(strip_option, into))]
    pub symbol: Option<String>,

    /// Exact asset type
    #[builder(default, setter(strip_option))]
    pub asset_type: Option<AssetType>,
}

impl AssetQuery {
    /// Check if an asset matches this query
    pub fn matches(&self, asset: &Asset) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if asset.id != id {
                return false;
            }
        }

        // Check symbol filter
        if let Some(ref symbol) = self.symbol {
            if !symbol.eq_ignore_ascii_case(&asset.symbol) {
                return false;
            }
        }

        // Check asset type filter
        if let Some(asset_type) = &self.asset_type {
            if asset.asset_type != *asset_type {
                return false;
            }
        }

        true
    }
}

/// Query builder for filtering instances (list operations)
///
/// Use this for get_instrument operations where you want to specify
/// exact matches for venue, base asset, quote asset, etc.
///
/// # Examples
/// ```ignore
/// // Find BTCUSDT perpetual on Binance
/// let query = InstrumentQuery::builder()
///     .venue(VenueName::BinanceUsdm)
///     .base_asset_symbol("BTC")
///     .quote_asset_symbol("USDT")
///     .instrument_type(InstrumentType::Perpetual)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct InstrumentQuery {
    /// Exact instrument id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,

    /// Exact venue match
    #[builder(default, setter(strip_option))]
    pub venue: Option<VenueName>,

    /// Exact venue symbol
    #[builder(default, setter(strip_option, into))]
    pub venue_symbol: Option<String>,

    /// Exact base asset (using Arc<Asset> for precise matching)
    #[builder(default, setter(strip_option))]
    #[serde(skip)] // Can't serialize Arc<Asset>
    pub base_asset: Option<Arc<Asset>>,

    /// Base asset symbol (case-insensitive, fallback if base_asset not provided)
    #[builder(default, setter(strip_option, into))]
    pub base_asset_symbol: Option<String>,

    /// Exact quote asset (using Arc<Asset> for precise matching)
    #[builder(default, setter(strip_option))]
    #[serde(skip)]
    pub quote_asset: Option<Arc<Asset>>,

    /// Quote asset symbol (case-insensitive, fallback if quote_asset not provided)
    #[builder(default, setter(strip_option, into))]
    pub quote_asset_symbol: Option<String>,

    /// Exact margin asset (using Arc<Asset> for precise matching)
    #[builder(default, setter(strip_option))]
    #[serde(skip)]
    pub margin_asset: Option<Arc<Asset>>,

    /// Margin asset symbol (case-insensitive, fallback if margin_asset not provided)
    #[builder(default, setter(strip_option, into))]
    pub margin_asset_symbol: Option<String>,

    /// Exact instrument type
    #[builder(default, setter(strip_option))]
    pub instrument_type: Option<InstrumentType>,

    /// Exact synthetic flag
    #[builder(default, setter(strip_option))]
    pub synthetic: Option<bool>,

    /// Exact status
    #[builder(default, setter(strip_option))]
    pub status: Option<InstrumentStatus>,
}

impl InstrumentQuery {
    /// Check if an instrument matches this query
    pub fn matches(&self, instrument: &Instrument) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if instrument.id != id {
                return false;
            }
        }

        // Check venue filter
        if let Some(ref venue) = self.venue {
            if instrument.venue.name != *venue {
                return false;
            }
        }

        // Check venue symbol filter
        if let Some(ref venue_symbol) = self.venue_symbol {
            if !venue_symbol.eq_ignore_ascii_case(&instrument.venue_symbol) {
                return false;
            }
        }

        // Check base asset filter
        if let Some(ref base_asset) = self.base_asset {
            if instrument.base_asset.id != base_asset.id {
                return false;
            }
        } else if let Some(ref symbol) = self.base_asset_symbol {
            if !symbol.eq_ignore_ascii_case(&instrument.base_asset.symbol) {
                return false;
            }
        }

        // Check quote asset filter
        if let Some(ref quote_asset) = self.quote_asset {
            if instrument.quote_asset.id != quote_asset.id {
                return false;
            }
        } else if let Some(ref symbol) = self.quote_asset_symbol {
            if !symbol.eq_ignore_ascii_case(&instrument.quote_asset.symbol) {
                return false;
            }
        }

        // Check margin asset filter
        if let Some(ref margin_asset) = self.margin_asset {
            if instrument.margin_asset.id != margin_asset.id {
                return false;
            }
        } else if let Some(ref symbol) = self.margin_asset_symbol {
            if !symbol.eq_ignore_ascii_case(&instrument.margin_asset.symbol) {
                return false;
            }
        }

        // Check instrument type filter
        if let Some(instrument_type) = self.instrument_type {
            if instrument.instrument_type != instrument_type {
                return false;
            }
        }

        // Check synthetic filter
        if let Some(synthetic) = self.synthetic {
            if instrument.synthetic != synthetic {
                return false;
            }
        }

        // Check status filter
        if let Some(status) = self.status {
            if instrument.status != status {
                return false;
            }
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.venue.is_none()
            && self.base_asset.is_none()
            && self.base_asset_symbol.is_none()
            && self.quote_asset.is_none()
            && self.quote_asset_symbol.is_none()
            && self.margin_asset.is_none()
            && self.margin_asset_symbol.is_none()
            && self.instrument_type.is_none()
            && self.synthetic.is_none()
            && self.status.is_none()
    }
}

/// Query builder for filtering instruments (list operations)
///
/// Empty vectors mean "no filter" (include all).
/// Use this for list_instruments operations where you want to
/// specify multiple options for each criteria.
///
/// # Examples
/// ```ignore
/// // Query BTC and ETH instruments on multiple venues
/// let query = InstrumentListQuery::builder()
///     .venues(vec![VenueName::BinanceSpot, VenueName::BinanceUsdm])
///     .base_asset_symbols(vec!["BTC", "ETH"])
///     .instrument_types(vec![InstrumentType::Spot, InstrumentType::Perpetual])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct InstrumentListQuery {
    /// Filter by ids
    #[builder(default, setter(strip_option))]
    pub ids: Option<Vec<Uuid>>,

    /// Filter by venue
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub venues: Vec<VenueName>,

    /// Filter by venue symbol (case-insensitive)
    #[builder(default, setter(strip_option, into))]
    pub venue_symbol: Option<String>,

    /// Filter by base asset (using Arc<Asset> for precise matching)
    /// Empty = no filter
    #[builder(default, setter(strip_option))]
    #[serde(skip)] // Can't serialize Arc<Asset>
    pub base_assets: Option<Vec<Arc<Asset>>>,

    /// Filter by base asset symbols (case-insensitive, fallback if base_assets not provided)
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub base_asset_symbols: Vec<String>,

    /// Filter by quote asset (using Arc<Asset> for precise matching)
    /// Empty = no filter
    #[builder(default, setter(strip_option))]
    #[serde(skip)]
    pub quote_assets: Option<Vec<Arc<Asset>>>,

    /// Filter by quote asset symbols (case-insensitive, fallback if quote_assets not provided)
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub quote_asset_symbols: Vec<String>,

    /// Filter by margin asset (using Arc<Asset> for precise matching)
    /// Empty = no filter
    #[builder(default, setter(strip_option))]
    #[serde(skip)]
    pub margin_assets: Option<Vec<Arc<Asset>>>,

    /// Filter by margin asset symbols (case-insensitive, fallback if margin_assets not provided)
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub margin_asset_symbols: Vec<String>,

    /// Filter by instrument type
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub instrument_types: Vec<InstrumentType>,

    /// Filter by synthetic flag
    /// None = no filter, Some(true) = only synthetic, Some(false) = only real
    #[builder(default)]
    #[serde(default)]
    pub synthetic: Option<bool>,

    /// Filter by status
    /// None = no filter
    #[builder(default)]
    #[serde(default)]
    pub status: Option<InstrumentStatus>,
}

impl InstrumentListQuery {
    /// Check if an instrument matches this query
    pub fn matches(&self, instrument: &Instrument) -> bool {
        // Check ids filter
        if let Some(ref ids) = self.ids {
            if !ids.contains(&instrument.id) {
                return false;
            }
        }

        // Check venue filter
        if !self.venues.is_empty() && !self.venues.contains(&instrument.venue.name) {
            return false;
        }

        // Check venue symbol filter
        if let Some(ref venue_symbol) = self.venue_symbol {
            if !venue_symbol.eq_ignore_ascii_case(&instrument.venue_symbol) {
                return false;
            }
        }

        // Check base asset - prefer Arc<Asset> over symbols
        if let Some(ref assets) = self.base_assets {
            if !assets.is_empty() && !assets.iter().any(|a| a.id == instrument.base_asset.id) {
                return false;
            }
        } else if !self.base_asset_symbols.is_empty()
            && !self
                .base_asset_symbols
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&instrument.base_asset.symbol))
        {
            return false;
        }

        // Check quote asset - prefer Arc<Asset> over symbols
        if let Some(ref assets) = self.quote_assets {
            if !assets.is_empty() && !assets.iter().any(|a| a.id == instrument.quote_asset.id) {
                return false;
            }
        } else if !self.quote_asset_symbols.is_empty()
            && !self
                .quote_asset_symbols
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&instrument.quote_asset.symbol))
        {
            return false;
        }

        // Check margin asset - prefer Arc<Asset> over symbols
        if let Some(ref assets) = self.margin_assets {
            if !assets.is_empty() && !assets.iter().any(|a| a.id == instrument.margin_asset.id) {
                return false;
            }
        } else if !self.margin_asset_symbols.is_empty()
            && !self
                .margin_asset_symbols
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&instrument.margin_asset.symbol))
        {
            return false;
        }

        // Check instrument type filter
        if !self.instrument_types.is_empty() && !self.instrument_types.contains(&instrument.instrument_type) {
            return false;
        }

        // Check synthetic flag
        if let Some(is_synthetic) = self.synthetic {
            if instrument.synthetic != is_synthetic {
                return false;
            }
        }

        // Check status
        if let Some(ref status) = self.status {
            if &instrument.status != status {
                return false;
            }
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.venues.is_empty()
            && self.base_assets.as_ref().map_or(true, |v| v.is_empty())
            && self.base_asset_symbols.is_empty()
            && self.quote_assets.as_ref().map_or(true, |v| v.is_empty())
            && self.quote_asset_symbols.is_empty()
            && self.margin_assets.as_ref().map_or(true, |v| v.is_empty())
            && self.margin_asset_symbols.is_empty()
            && self.instrument_types.is_empty()
            && self.synthetic.is_none()
            && self.status.is_none()
    }

    /// Build query using only base/quote asset symbols for convenience
    pub fn from_symbols(
        base_symbols: impl IntoIterator<Item = impl Into<String>>,
        quote_symbols: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::builder()
            .base_asset_symbols(base_symbols.into_iter().map(Into::into).collect::<Vec<_>>())
            .quote_asset_symbols(quote_symbols.into_iter().map(Into::into).collect::<Vec<_>>())
            .build()
    }

    /// Build query using resolved assets
    pub fn from_assets(
        base_assets: impl IntoIterator<Item = Arc<Asset>>,
        quote_assets: impl IntoIterator<Item = Arc<Asset>>,
    ) -> Self {
        Self::builder()
            .base_assets(base_assets.into_iter().collect::<Vec<_>>())
            .quote_assets(quote_assets.into_iter().collect::<Vec<_>>())
            .build()
    }
}

/// Query builder for filtering venues (list operations)
///
/// Empty vectors mean "no filter" (include all).
/// Use this for list_venues operations where you want to
/// specify multiple options for each criteria.
///
/// # Examples
/// ```ignore
/// // Find all Binance venues
/// let query = VenueListQuery::builder()
///     .names(vec![VenueName::BinanceSpot, VenueName::BinanceUsdmFutures])
///     .build();
/// let venues = persistence.list_venues(&query).await?;
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct VenueListQuery {
    /// Filter by venue names
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub names: Vec<VenueName>,

    /// Filter by venue type
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub venue_types: Vec<VenueType>,
}

impl VenueListQuery {
    /// Check if a venue matches this query
    pub fn matches(&self, venue: &Venue) -> bool {
        // Check names filter
        if !self.names.is_empty() && !self.names.contains(&venue.name) {
            return false;
        }

        // Check venue type filter
        if !self.venue_types.is_empty() && !self.venue_types.contains(&venue.venue_type) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.venue_types.is_empty()
    }
}

/// Query for finding a single venue with exact criteria
///
/// Use this for get_venue operations where you want to specify
/// exact matches for name, venue type, etc.
///
/// # Examples
/// ```ignore
/// // Find Binance spot venue
/// let query = VenueQuery::builder()
///     .name(VenueName::BinanceSpot)
///     .venue_type(VenueType::Spot)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct VenueQuery {
    /// Exact venue id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,

    /// Exact venue name
    #[builder(default, setter(strip_option))]
    pub name: Option<VenueName>,

    /// Exact venue type
    #[builder(default, setter(strip_option))]
    pub venue_type: Option<VenueType>,
}

impl VenueQuery {
    /// Check if a venue matches this query
    pub fn matches(&self, venue: &Venue) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if venue.id != id {
                return false;
            }
        }

        // Check name filter
        if let Some(name) = self.name {
            if venue.name != name {
                return false;
            }
        }

        // Check venue type filter
        if let Some(venue_type) = &self.venue_type {
            if venue.venue_type != *venue_type {
                return false;
            }
        }

        true
    }
}

/// Query for finding a single instance with exact criteria
///
/// Use this for get_instance operations where you want to specify
/// exact matches for name, instance type, etc.
///
/// # Examples
/// ```ignore
/// // Find prod live instance
/// let query = InstanceQuery::builder()
///     .name("prod")
///     .instance_type(InstanceType::Live)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct InstanceQuery {
    /// Exact instance name
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,

    /// Exact instance type
    #[builder(default, setter(strip_option))]
    pub instance_type: Option<InstanceType>,
}

impl InstanceQuery {
    /// Check if an instance matches this query
    pub fn matches(&self, instance: &Instance) -> bool {
        // Check name filter
        if let Some(ref name) = self.name {
            if instance.name != *name {
                return false;
            }
        }

        // Check instance type filter
        if let Some(instance_type) = self.instance_type {
            if instance.instance_type != instance_type {
                return false;
            }
        }

        true
    }
}

/// Query builder for filtering pipelines (list operations)
///
/// Empty vectors/None values mean "no filter" (include all).
///
/// # Examples
/// ```ignore
/// // Find pipelines by name pattern
/// let query = PipelineQuery::builder()
///     .names(vec!["insights_v2".to_string()])
///     .build();
/// let pipelines = persistence.query_pipelines(&query).await?;
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct PipelineListQuery {
    /// Filter by pipeline names (exact match)
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub names: Vec<String>,
}

impl PipelineListQuery {
    /// Check if a pipeline matches this query
    pub fn matches(&self, pipeline: &Pipeline) -> bool {
        // Check names filter
        if !self.names.is_empty() && !self.names.contains(&pipeline.name) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Query for finding a single pipeline with exact criteria
///
/// Use this for get_pipeline operations where you want to specify
/// exact matches for name, etc.
///
/// # Examples
/// ```ignore
/// // Find insights_v2 pipeline
/// let query = PipelineQuery::builder()
///     .name("insights_v2")
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct PipelineQuery {
    /// Exact pipeline id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,

    /// Exact pipeline name
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,
}

impl PipelineQuery {
    /// Check if a pipeline matches this query
    pub fn matches(&self, pipeline: &Pipeline) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if pipeline.id != id {
                return false;
            }
        }

        // Check name filter
        if let Some(ref name) = self.name {
            if pipeline.name != *name {
                return false;
            }
        }

        true
    }
}

/// Query builder for filtering instances (list operations)
///
/// Empty vectors mean "no filter" (include all).
/// Use this for list_instances operations where you want to
/// specify multiple options for each criteria.
///
/// # Examples
/// ```ignore
/// // Query live and backtest instances
/// let query = InstanceListQuery::builder()
///     .names(vec!["prod", "backtest"])
///     .instance_types(vec![InstanceType::Live, InstanceType::Backtest])
///     .build();
/// let instances = persistence.list_instances(&query).await?;
/// ```
#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct InstanceListQuery {
    /// Filter by instance names (exact match)
    /// Empty = no filter
    #[builder(default, setter(into))]
    pub names: Vec<String>,

    /// Filter by instance type
    /// Empty = no filter
    #[builder(default, setter(into))]
    pub instance_types: Vec<InstanceType>,
}

impl InstanceListQuery {
    /// Check if an instance matches this query
    pub fn matches(&self, instance: &Instance) -> bool {
        // Check names filter
        if !self.names.is_empty() && !self.names.contains(&instance.name) {
            return false;
        }

        // Check instance type filter
        if !self.instance_types.is_empty() && !self.instance_types.contains(&instance.instance_type) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.instance_types.is_empty()
    }
}

/// Query builder for filtering features (single exact match)
///
/// Use this for get_feature operations where you want to specify
/// exact match for feature id.
///
/// # Examples
/// ```ignore
/// // Find feature by id
/// let query = FeatureQuery::builder()
///     .id("trade_price")
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct FeatureQuery {
    /// Exact feature id
    #[builder(default, setter(into))]
    pub id: String,
}

/// Query builder for filtering features (list operations)
///
/// Empty vectors mean "no filter" (include all).
/// Use this for list_features operations where you want to
/// specify multiple options for each criteria.
///
/// # Examples
/// ```ignore
/// // Query specific features
/// let query = FeatureListQuery::builder()
///     .ids(vec!["trade_price", "trade_quantity"])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct FeatureListQuery {
    /// Filter by feature ids
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub ids: Vec<String>,
}

impl FeatureListQuery {
    /// Check if a feature id matches this query
    pub fn matches(&self, feature_id: &str) -> bool {
        // Check ids filter
        if !self.ids.is_empty() && !self.ids.contains(&feature_id.to_string()) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Query builder for filtering accounts
///
/// Use this for get_account operations where you want to specify
/// exact matches for id.
///
/// # Examples
/// ```ignore
/// // Find account by id
/// let query = AccountQuery::builder()
///     .id(uuid)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct AccountQuery {
    /// Exact account id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,
}

impl AccountQuery {
    /// Check if an account matches this query
    pub fn matches(&self, account: &Account) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if account.id != id {
                return false;
            }
        }

        true
    }
}

/// Query builder for filtering accounts (list operations)
///
/// Use this for list_accounts operations where you want to filter
/// by ids.
///
/// # Examples
/// ```ignore
/// // Find accounts by ids
/// let query = AccountListQuery::builder()
///     .ids(vec![uuid1, uuid2])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct AccountListQuery {
    /// Filter by specific account ids
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

impl AccountListQuery {
    /// Check if an account matches this query
    pub fn matches(&self, account: &Account) -> bool {
        // Check ids filter
        if !self.ids.is_empty() && !self.ids.contains(&account.id) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Query builder for filtering strategies
///
/// Use this for get_strategy operations where you want to specify
/// exact matches for id or name.
///
/// # Examples
/// ```ignore
/// // Find strategy by id
/// let query = StrategyQuery::builder()
///     .id(uuid)
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct StrategyQuery {
    /// Exact strategy id
    #[builder(default, setter(strip_option))]
    pub id: Option<Uuid>,

    /// Exact strategy name
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,
}

impl StrategyQuery {
    /// Check if a strategy matches this query
    pub fn matches(&self, strategy: &Strategy) -> bool {
        // Check id filter
        if let Some(id) = self.id {
            if strategy.id != id {
                return false;
            }
        }

        // Check name filter
        if let Some(ref name) = self.name {
            if strategy.name != *name {
                return false;
            }
        }

        true
    }
}

/// Query builder for filtering strategies (list operations)
///
/// Use this for list_strategies operations where you want to filter
/// by ids or names.
///
/// # Examples
/// ```ignore
/// // Find strategies by names
/// let query = StrategyListQuery::builder()
///     .names(vec!["strategy1", "strategy2"])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct StrategyListQuery {
    /// Filter by specific strategy ids
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub ids: Vec<Uuid>,

    /// Filter by specific strategy names
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub names: Vec<String>,
}

impl StrategyListQuery {
    /// Check if a strategy matches this query
    pub fn matches(&self, strategy: &Strategy) -> bool {
        // Check ids filter
        if !self.ids.is_empty() && !self.ids.contains(&strategy.id) {
            return false;
        }

        // Check names filter
        if !self.names.is_empty() && !self.names.contains(&strategy.name) {
            return false;
        }

        true
    }

    /// Returns true if this query has no filters (matches all)
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty() && self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::UtcDateTime;

    #[test]
    fn test_asset_list_query_empty_matches_all() {
        let query = AssetListQuery::default();
        assert!(query.is_empty());

        let asset = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };
        assert!(query.matches(&asset));
    }

    #[test]
    fn test_asset_list_query_symbol_filter() {
        let query = AssetListQuery::builder()
            .symbols(vec!["BTC".to_string(), "ETH".to_string()])
            .build();

        let btc = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };

        let sol = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };

        assert!(query.matches(&btc));
        assert!(!query.matches(&sol));
    }

    #[test]
    fn test_asset_query() {
        let query = AssetQuery::builder().symbol("BTC").asset_type(AssetType::Crypto).build();

        let btc = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };
        assert!(query.matches(&btc));

        let eth = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };
        assert!(!query.matches(&eth));
    }
}
