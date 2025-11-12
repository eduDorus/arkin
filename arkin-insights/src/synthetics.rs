use std::{collections::HashSet, sync::Arc};

use rust_decimal::Decimal;
use tracing::info;

use arkin_core::prelude::*;

use crate::config::{FeatureConfig, GroupBy, InstrumentSelector, PipelineConfig};

/// Generates synthetic instruments based on pipeline feature configuration
pub struct SyntheticGenerator {}

impl SyntheticGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate all synthetic instruments required by the pipeline configuration
    pub async fn generate(
        &self,
        persistence: &Arc<dyn PersistenceReader>,
        config: &PipelineConfig,
    ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        info!(target: "insights", "Generating synthetic instruments from pipeline config");

        let mut all_synthetics = Vec::new();

        // Step 1: Generate base asset synthetics from Grouped features
        let base_synthetics = self.generate_base_synthetics(persistence, config).await?;
        info!(target: "insights", "Generated {} base synthetic instruments", base_synthetics.len());
        all_synthetics.extend(base_synthetics);

        // Step 2: Generate index synthetics from Index features
        let index_synthetics = self.generate_index_synthetics(persistence, config, &all_synthetics).await?;
        info!(target: "insights", "Generated {} index synthetic instruments", index_synthetics.len());
        all_synthetics.extend(index_synthetics);

        info!(target: "insights", "Total synthetic instruments created: {}", all_synthetics.len());

        Ok(all_synthetics)
    }

    /// Generate base asset synthetic instruments from Grouped features
    async fn generate_base_synthetics(
        &self,
        persistence: &Arc<dyn PersistenceReader>,
        config: &PipelineConfig,
    ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        let mut synthetics = Vec::new();

        // Get Index venue
        let index_venue = persistence.get_venue_by_name(&VenueName::Index).await?;

        // Get reference currency asset (e.g., USD)
        let ref_asset_query = AssetQuery::builder().symbols(vec![config.reference_currency.clone()]).build();
        let ref_asset = persistence
            .query_assets(&ref_asset_query)
            .await?
            .into_iter()
            .next()
            .ok_or(PersistenceError::NotFound)?;

        let timestamp = time::OffsetDateTime::now_utc();

        // Scan all features for Grouped aggregation types
        for feature in &config.features {
            if let Some(grouped_config) = Self::extract_grouped_config(feature) {
                let (selector, group_by) = grouped_config;

                // Merge feature selector with global selector by combining constraints
                // (both base_asset and quote_asset constraints must be satisfied)
                let mut merged_base_assets = selector.base_asset.clone();
                if merged_base_assets.is_empty() {
                    merged_base_assets = config.global_instrument_selector.base_asset.clone();
                }

                let mut merged_quote_assets = selector.quote_asset.clone();
                if merged_quote_assets.is_empty() {
                    merged_quote_assets = config.global_instrument_selector.quote_asset.clone();
                }

                let mut merged_instrument_types = selector.instrument_type.clone();
                if merged_instrument_types.is_empty() {
                    merged_instrument_types = config.global_instrument_selector.instrument_type.clone();
                }

                let mut merged_venues = selector.venue.clone();
                if merged_venues.is_empty() {
                    merged_venues = config.global_instrument_selector.venue.clone();
                }

                let merged_selector = InstrumentSelector {
                    base_asset: merged_base_assets,
                    quote_asset: merged_quote_assets,
                    instrument_type: merged_instrument_types,
                    venue: merged_venues,
                    symbols: selector.symbols.clone(),
                    synthetic: Some(false), // Always real instruments for grouping
                };

                // Query real instruments matching the merged selector
                let instrument_query = Self::build_instrument_query(&merged_selector);
                let real_instruments = persistence.query_instruments(&instrument_query).await?;

                if real_instruments.is_empty() {
                    continue;
                }

                // Track which synthetics we've already created
                let mut seen = HashSet::new();

                for inst in &real_instruments {
                    // Build grouping key based on configuration
                    let (key, venue_to_use) = if group_by.venue {
                        let k = if group_by.instrument_type {
                            (
                                inst.instrument_type.clone(),
                                inst.base_asset.symbol.clone(),
                                inst.venue.name.clone(),
                            )
                        } else {
                            (InstrumentType::Index, inst.base_asset.symbol.clone(), inst.venue.name.clone())
                        };
                        (k, inst.venue.clone())
                    } else {
                        let k = if group_by.instrument_type {
                            (inst.instrument_type.clone(), inst.base_asset.symbol.clone(), VenueName::Index)
                        } else {
                            (InstrumentType::Index, inst.base_asset.symbol.clone(), VenueName::Index)
                        };
                        (k, index_venue.clone())
                    };

                    if seen.insert(key.clone()) {
                        // Build symbol based on whether we group by instrument type
                        let symbol = if group_by.instrument_type {
                            format!(
                                "syn-{}-{}-{}@{}",
                                key.0.to_string().to_lowercase(),
                                key.1.to_lowercase(),
                                config.reference_currency.to_lowercase(),
                                venue_to_use.name.to_string().to_lowercase()
                            )
                        } else {
                            format!(
                                "syn-{}-{}@{}",
                                key.1.to_lowercase(),
                                config.reference_currency.to_lowercase(),
                                venue_to_use.name.to_string().to_lowercase()
                            )
                        };

                        let synthetic = Self::create_instrument(
                            symbol,
                            venue_to_use,
                            inst.base_asset.clone(),
                            ref_asset.clone(),
                            if group_by.instrument_type {
                                inst.instrument_type.clone()
                            } else {
                                InstrumentType::Index
                            },
                            timestamp,
                        );

                        synthetics.push(synthetic);
                    }
                }
            }
        }

        Ok(synthetics)
    }

    /// Generate index synthetic instruments from Index features
    async fn generate_index_synthetics(
        &self,
        persistence: &Arc<dyn PersistenceReader>,
        config: &PipelineConfig,
        base_synthetics: &[Arc<Instrument>],
    ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        let mut synthetics = Vec::new();

        // Get Index venue
        let index_venue = persistence.get_venue_by_name(&VenueName::Index).await?;

        // Get reference currency asset
        let ref_asset_query = AssetQuery::builder().symbols(vec![config.reference_currency.clone()]).build();
        let ref_asset = persistence
            .query_assets(&ref_asset_query)
            .await?
            .into_iter()
            .next()
            .ok_or(PersistenceError::NotFound)?;

        let timestamp = time::OffsetDateTime::now_utc();

        // Scan all features for Index aggregation types
        for feature in &config.features {
            if let Some((filter, outputs)) = Self::extract_index_config(feature) {
                // Filter base synthetics to match the index filter
                let matching_synthetics: Vec<_> = base_synthetics
                    .iter()
                    .filter(|inst| Self::matches_filter(inst, filter))
                    .collect();

                if matching_synthetics.is_empty() {
                    continue;
                }

                // Create one index instrument per output
                for output_name in outputs {
                    let symbol = format!("{}@{}", output_name, index_venue.name.to_string());

                    let synthetic = Self::create_instrument(
                        symbol,
                        index_venue.clone(),
                        ref_asset.clone(),
                        ref_asset.clone(),
                        InstrumentType::Index,
                        timestamp,
                    );

                    synthetics.push(synthetic);
                }
            }
        }

        Ok(synthetics)
    }

    /// Extract Grouped configuration from a feature
    fn extract_grouped_config(feature: &FeatureConfig) -> Option<(&InstrumentSelector, &GroupBy)> {
        use FeatureConfig;

        match feature {
            FeatureConfig::Range(c) => Some((&c.instrument_selector, &c.group_by)),
            FeatureConfig::DualRange(c) => Some((&c.instrument_selector, &c.group_by)),
            FeatureConfig::TwoValue(c) => Some((&c.instrument_selector_1, &c.group_by)),
            _ => None,
        }
    }

    /// Extract Index configuration from a feature
    fn extract_index_config(feature: &FeatureConfig) -> Option<(&InstrumentSelector, &[String])> {
        match feature {
            FeatureConfig::Range(c) => Some((&c.instrument_selector, &c.output)),
            FeatureConfig::DualRange(c) => Some((&c.instrument_selector, &c.output)),
            _ => None,
        }
    }

    /// Build InstrumentQuery from InstrumentSelector
    pub fn build_instrument_query(selector: &InstrumentSelector) -> InstrumentQuery {
        InstrumentQuery::builder()
            .base_asset_symbols(
                selector
                    .base_asset
                    .iter()
                    .map(|s: &String| s.to_uppercase())
                    .collect::<Vec<_>>(),
            )
            .quote_asset_symbols(
                selector
                    .quote_asset
                    .iter()
                    .map(|s: &String| s.to_uppercase())
                    .collect::<Vec<_>>(),
            )
            .instrument_types(selector.instrument_type.clone())
            .venues(selector.venue.clone())
            .synthetic(selector.synthetic)
            .build()
    }

    /// Check if an instrument matches a selector
    fn matches_filter(inst: &Instrument, selector: &InstrumentSelector) -> bool {
        // Check base asset
        if !selector.base_asset.is_empty()
            && !selector
                .base_asset
                .iter()
                .any(|s: &String| s.eq_ignore_ascii_case(&inst.base_asset.symbol))
        {
            return false;
        }

        // Check quote asset
        if !selector.quote_asset.is_empty()
            && !selector
                .quote_asset
                .iter()
                .any(|s: &String| s.eq_ignore_ascii_case(&inst.quote_asset.symbol))
        {
            return false;
        }

        // Check instrument type
        if !selector.instrument_type.is_empty() && !selector.instrument_type.contains(&inst.instrument_type) {
            return false;
        }

        // Check venue
        if !selector.venue.is_empty() && !selector.venue.contains(&inst.venue.name) {
            return false;
        }

        // Check symbols (exact match)
        if !selector.symbols.is_empty() && !selector.symbols.contains(&inst.symbol) {
            return false;
        }

        // Check synthetic flag
        if let Some(synthetic) = selector.synthetic {
            if inst.synthetic != synthetic {
                return false;
            }
        }

        true
    }

    /// Create a synthetic instrument
    fn create_instrument(
        symbol: String,
        venue: Arc<Venue>,
        base_asset: Arc<Asset>,
        quote_asset: Arc<Asset>,
        instrument_type: InstrumentType,
        timestamp: time::OffsetDateTime,
    ) -> Arc<Instrument> {
        Arc::new(
            Instrument::builder()
                .symbol(symbol.clone())
                .venue_symbol(symbol)
                .venue(venue)
                .base_asset(base_asset)
                .quote_asset(quote_asset.clone())
                .margin_asset(quote_asset)
                .instrument_type(instrument_type)
                .synthetic(true)
                .maturity(None)
                .strike(None)
                .option_type(None)
                .contract_size(Decimal::ONE)
                .price_precision(8)
                .quantity_precision(8)
                .base_precision(8)
                .quote_precision(8)
                .tick_size(Decimal::new(1, 8))
                .lot_size(Decimal::new(1, 8))
                .status(InstrumentStatus::Trading)
                .created(timestamp.into())
                .updated(timestamp.into())
                .build(),
        )
    }
}
