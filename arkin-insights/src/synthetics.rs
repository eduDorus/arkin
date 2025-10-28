use std::{collections::HashSet, sync::Arc};

use rust_decimal::Decimal;
use tracing::info;

use arkin_core::prelude::*;

use crate::config::{AggregationType, FeatureConfig, GroupBy, InstrumentFilter, PipelineConfig};

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
                let (filter, group_by) = grouped_config;

                // Merge feature filter with global filter
                let merged_filter = filter.merge_with_global(&config.instrument_filter);

                // Query real instruments matching the merged filter
                let instrument_query = Self::build_instrument_query(&merged_filter);
                let real_instruments = persistence.query_instruments(&instrument_query).await?;

                if real_instruments.is_empty() {
                    continue;
                }

                // Determine venue string for synthetic symbol
                let venue_str = group_by.venue.as_ref().map(|v| v.exchange_name()).unwrap_or("index");

                // Generate synthetics based on grouping strategy
                if group_by.instrument_type {
                    // Group by instrument type + base asset
                    let mut seen = HashSet::new();

                    for inst in &real_instruments {
                        let key = (inst.instrument_type.clone(), inst.base_asset.symbol.clone());
                        if seen.insert(key.clone()) {
                            let symbol = format!(
                                "syn-{}-{}-{}@{}",
                                inst.instrument_type.to_string().to_lowercase(),
                                inst.base_asset.symbol.to_lowercase(),
                                config.reference_currency.to_lowercase(),
                                venue_str
                            );

                            let synthetic = Self::create_instrument(
                                symbol,
                                index_venue.clone(),
                                inst.base_asset.clone(),
                                ref_asset.clone(),
                                inst.instrument_type.clone(),
                                timestamp,
                            );

                            synthetics.push(synthetic);
                        }
                    }
                } else {
                    // Group by base asset only
                    let mut seen = HashSet::new();

                    for inst in &real_instruments {
                        if seen.insert(inst.base_asset.symbol.clone()) {
                            let symbol = format!(
                                "syn-{}-{}@{}",
                                inst.base_asset.symbol.to_lowercase(),
                                config.reference_currency.to_lowercase(),
                                venue_str
                            );

                            let synthetic = Self::create_instrument(
                                symbol,
                                index_venue.clone(),
                                inst.base_asset.clone(),
                                ref_asset.clone(),
                                InstrumentType::Index,
                                timestamp,
                            );

                            synthetics.push(synthetic);
                        }
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
                    let symbol = format!("{}@index", output_name);

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
    fn extract_grouped_config(feature: &FeatureConfig) -> Option<(&InstrumentFilter, &GroupBy)> {
        use FeatureConfig;

        match feature {
            FeatureConfig::Range(c) => {
                if let AggregationType::Grouped { filter, group_by } = &c.aggregation_type {
                    Some((filter, group_by))
                } else {
                    None
                }
            }
            FeatureConfig::DualRange(c) => {
                if let AggregationType::Grouped { filter, group_by } = &c.aggregation_type {
                    Some((filter, group_by))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract Index configuration from a feature
    fn extract_index_config(feature: &FeatureConfig) -> Option<(&InstrumentFilter, &[String])> {
        match feature {
            FeatureConfig::Range(c) => {
                if let AggregationType::Index { filter } = &c.aggregation_type {
                    Some((filter, &c.output))
                } else {
                    None
                }
            }
            FeatureConfig::DualRange(c) => {
                if let AggregationType::Index { filter } = &c.aggregation_type {
                    Some((filter, &c.output))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Build InstrumentQuery from InstrumentFilter
    pub fn build_instrument_query(filter: &InstrumentFilter) -> InstrumentQuery {
        InstrumentQuery::builder()
            .base_asset_symbols(filter.base_asset.iter().map(|s| s.to_uppercase()).collect::<Vec<_>>())
            .quote_asset_symbols(filter.quote_asset.iter().map(|s| s.to_uppercase()).collect::<Vec<_>>())
            .instrument_types(filter.instrument_type.clone())
            .venues(filter.venue.clone())
            .synthetic(filter.synthetic)
            .build()
    }

    /// Check if an instrument matches a filter
    fn matches_filter(inst: &Instrument, filter: &InstrumentFilter) -> bool {
        // Check base asset
        if !filter.base_asset.is_empty()
            && !filter
                .base_asset
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&inst.base_asset.symbol))
        {
            return false;
        }

        // Check quote asset
        if !filter.quote_asset.is_empty()
            && !filter
                .quote_asset
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&inst.quote_asset.symbol))
        {
            return false;
        }

        // Check instrument type
        if !filter.instrument_type.is_empty() && !filter.instrument_type.contains(&inst.instrument_type) {
            return false;
        }

        // Check venue
        if !filter.venue.is_empty() && !filter.venue.contains(&inst.venue.name) {
            return false;
        }

        // Check synthetic flag
        if let Some(synthetic) = filter.synthetic {
            if inst.synthetic != synthetic {
                return false;
            }
        }

        // Check venue for @index synthetics
        if inst.symbol.ends_with("@index") && inst.venue.name != VenueName::Index {
            return false;
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
