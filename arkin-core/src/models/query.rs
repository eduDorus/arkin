use std::sync::Arc;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{Asset, AssetType, Instrument, InstrumentStatus, InstrumentType, VenueName};

/// Query builder for filtering assets
///
/// Empty vectors mean "no filter" (include all).
/// Use the builder pattern for ergonomic construction.
///
/// # Examples
/// ```ignore
/// // Query crypto and stablecoin assets with specific symbols
/// let query = AssetQuery::builder()
///     .symbols(vec!["BTC", "ETH", "USDT"])
///     .asset_types(vec![AssetType::Crypto, AssetType::Stablecoin])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct AssetQuery {
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

impl AssetQuery {
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

/// Query builder for filtering instruments with support for asset-based filtering
///
/// Empty vectors/None values mean "no filter" (include all).
/// Supports both string-based and Arc<Asset>-based filtering.
///
/// # Examples
/// ```ignore
/// // Query BTC/ETH perpetuals on Binance, non-synthetic
/// let query = InstrumentQuery::builder()
///     .venues(vec![VenueName::BinanceUsdmFutures])
///     .base_asset_symbols(vec!["BTC", "ETH"])
///     .quote_asset_symbols(vec!["USDT"])
///     .instrument_types(vec![InstrumentType::Perpetual])
///     .synthetic(Some(false))
///     .build();
///
/// // Using resolved assets for more precise filtering
/// let btc = persistence.get_asset_by_symbol("BTC").await?;
/// let eth = persistence.get_asset_by_symbol("ETH").await?;
/// let query = InstrumentQuery::builder()
///     .base_assets(vec![btc, eth])
///     .build();
/// ```
#[derive(Debug, Clone, Default, TypedBuilder, Serialize, Deserialize)]
pub struct InstrumentQuery {
    /// Filter by venue
    /// Empty = no filter
    #[builder(default, setter(into))]
    #[serde(default)]
    pub venues: Vec<VenueName>,

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

impl InstrumentQuery {
    /// Check if an instrument matches this query
    pub fn matches(&self, instrument: &Instrument) -> bool {
        // Check venue filter
        if !self.venues.is_empty() && !self.venues.contains(&instrument.venue.name) {
            return false;
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::UtcDateTime;

    #[test]
    fn test_asset_query_empty_matches_all() {
        let query = AssetQuery::default();
        let asset = Asset {
            id: uuid::Uuid::new_v4(),
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            asset_type: AssetType::Crypto,
            created: UtcDateTime::now(),
            updated: UtcDateTime::now(),
        };
        assert!(query.matches(&asset));
        assert!(query.is_empty());
    }

    #[test]
    fn test_asset_query_symbol_filter() {
        let query = AssetQuery::builder()
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
    fn test_instrument_query_empty_matches_all() {
        let query = InstrumentQuery::default();
        assert!(query.is_empty());
    }
}
