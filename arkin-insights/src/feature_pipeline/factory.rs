use std::sync::Arc;

use arkin_core::{Instrument, PersistenceReader};
use tracing::info;

use crate::{
    config::{AggregationType, FeatureConfig, PipelineConfig},
    features::{
        DistributionType, DualRangeFeature, LagFeature, NormalizeFeature, QuantileTransformer, RangeFeature,
        RobustScaler, TwoValueFeature,
    },
    synthetics::SyntheticGenerator,
    Feature, InstrumentScope,
};

pub struct FeatureFactory {}

impl FeatureFactory {
    /// Create features from configuration
    /// This is the main entry point - it handles:
    /// 1. Querying real instruments from persistence using global filter → our universe
    /// 2. Generating synthetic instruments based on feature configs from that universe
    /// 3. Each feature filters the universe based on its own filter (no merging)
    /// 4. Building InstrumentScope mappings for each feature
    /// 5. Creating feature instances with their scopes
    pub async fn from_config(
        persistence: &Arc<dyn PersistenceReader>,
        pipeline_config: &PipelineConfig,
    ) -> Vec<Arc<dyn Feature>> {
        info!("FeatureFactory: Creating features from config");

        // Step 1: Query real instruments using global filter - this is our universe
        let mut instrument_query = SyntheticGenerator::build_instrument_query(&pipeline_config.instrument_filter);
        // Ensure we only get real instruments (not synthetics)
        instrument_query.synthetic = Some(false);

        let real_instruments = persistence
            .query_instruments(&instrument_query)
            .await
            .expect("Failed to query real instruments");
        info!("FeatureFactory: Loaded {} real instruments", real_instruments.len());

        // Step 2: Generate synthetic instruments based on pipeline config from our universe
        let generator = SyntheticGenerator::new();
        let synthetic_instruments = generator
            .generate(persistence, pipeline_config)
            .await
            .expect("Failed to generate synthetic instruments");
        info!(
            "FeatureFactory: Generated {} synthetic instruments",
            synthetic_instruments.len()
        );

        // Step 3: Create features - each feature filters the universe on its own preferences
        Self::create_features_internal(
            persistence,
            &pipeline_config.features,
            &real_instruments,
            &synthetic_instruments,
        )
        .await
    }

    /// Internal method to create feature instances from configs and instruments
    async fn create_features_internal(
        persistence: &Arc<dyn PersistenceReader>,
        configs: &[FeatureConfig],
        real_instruments: &[Arc<Instrument>],
        synthetic_instruments: &[Arc<Instrument>],
    ) -> Vec<Arc<dyn Feature>> {
        let mut features = Vec::new();

        for config in configs {
            match config {
                FeatureConfig::Lag(c) => {
                    // Validate that input, output, and lag have the same length
                    assert_eq!(c.input.len(), c.output.len(), "input and output must have the same length");
                    assert_eq!(c.input.len(), c.lag.len(), "input and lag must have the same length");

                    // Build scopes - feature filters the universe with its own filter
                    let scopes = Self::build_scopes(&c.aggregation_type, real_instruments, synthetic_instruments);

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            LagFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .lag(c.lag[i])
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .scopes(scopes.clone())
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::Range(c) => {
                    // Validate that input, output, and data have the same length
                    assert_eq!(c.input.len(), c.output.len(), "input and output must have the same length");
                    assert_eq!(c.input.len(), c.data.len(), "input and data must have the same length");

                    // Build scopes once for all range features with the same aggregation type
                    let scopes = Self::build_scopes(&c.aggregation_type, real_instruments, synthetic_instruments);
                    tracing::info!(
                        "Range feature with aggregation_type {:?} has {} scopes",
                        c.aggregation_type,
                        scopes.len()
                    );

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            RangeFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .data(c.data[i].clone())
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .scopes(scopes.clone())
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::DualRange(c) => {
                    // Validate that all arrays have the same length
                    assert_eq!(
                        c.input_1.len(),
                        c.input_2.len(),
                        "input_1 and input_2 must have the same length"
                    );
                    assert_eq!(c.input_1.len(), c.output.len(), "input_1 and output must have the same length");
                    assert_eq!(c.input_1.len(), c.data.len(), "input_1 and data must have the same length");

                    // Build scopes once for all dual range features with the same aggregation type
                    let scopes = Self::build_scopes(&c.aggregation_type, real_instruments, synthetic_instruments);

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            DualRangeFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .data(c.data[i].clone())
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .scopes(scopes.clone())
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::TwoValue(c) => {
                    // Validate that all arrays have the same length
                    assert_eq!(
                        c.input_1.len(),
                        c.input_2.len(),
                        "input_1 and input_2 must have the same length"
                    );
                    assert_eq!(c.input_1.len(), c.output.len(), "input_1 and output must have the same length");

                    // Build scopes once for all two value features with the same aggregation type
                    let scopes = Self::build_scopes(&c.aggregation_type, real_instruments, synthetic_instruments);

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            TwoValueFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .method(c.method.clone())
                                .fill_strategy(c.fill_strategy)
                                .scopes(scopes.clone())
                                .build(),
                        ) as Arc<dyn Feature>);
                    }
                }
                FeatureConfig::Normalize(c) => {
                    let transformer = QuantileTransformer::new(&c.data_location, &c.version, DistributionType::Normal);
                    let scaler = RobustScaler::new(&c.data_location, &c.version);

                    // Register input features
                    let mut input_features = Vec::new();
                    for input in &c.input {
                        input_features.push(persistence.get_feature_id(input).await);
                    }

                    // Register output feature
                    let output_feature = persistence.get_feature_id(&c.output).await;

                    // Build scopes for normalize feature
                    let scopes = Self::build_scopes(&c.aggregation_type, real_instruments, synthetic_instruments);

                    features.push(Arc::new(
                        NormalizeFeature::builder()
                            .input(input_features)
                            .output(output_feature)
                            .transformer(transformer)
                            .scaler(scaler)
                            .method(c.method.clone())
                            .scopes(scopes)
                            .build(),
                    ) as Arc<dyn Feature>);
                }
            }
        }

        features
    }

    /// Build instrument scopes based on aggregation type
    /// Each scope represents a collection of instruments that should be processed together
    /// The feature's filter is applied to the universe (real + synthetic instruments)
    fn build_scopes(
        aggregation_type: &AggregationType,
        real_instruments: &[Arc<Instrument>],
        synthetic_instruments: &[Arc<Instrument>],
    ) -> Vec<InstrumentScope> {
        match aggregation_type {
            // Instrument-level: each real instrument gets a 1:1 scope (read from itself, write to itself)
            AggregationType::Instrument { filter } => real_instruments
                .iter()
                .filter(|inst| Self::matches_filter(inst, filter))
                .map(|inst| InstrumentScope::single(Arc::clone(inst)))
                .collect(),

            // Grouped: each synthetic reads from multiple real instruments
            AggregationType::Grouped { filter, group_by } => {
                synthetic_instruments
                    .iter()
                    .filter(|inst| {
                        // Must be synthetic
                        if !inst.synthetic {
                            return false;
                        }

                        // Filter out index synthetics (don't start with syn-)
                        // Index synthetics have names like: index_notional_01m@index
                        // Base synthetics have names like: syn-btc-usd@index
                        if !inst.symbol.starts_with("syn-") {
                            return false;
                        }

                        // If feature specifies a venue, synthetic symbol must match that venue
                        // e.g., venue=Some(BinanceUsdmFutures) means symbol must contain "@binance"
                        if let Some(feature_venue) = group_by.venue {
                            let venue_suffix = format!("@{}", feature_venue.exchange_name().to_lowercase());
                            if !inst.symbol.to_lowercase().contains(&venue_suffix) {
                                return false;
                            }
                        } else {
                            // If no venue specified (global aggregation), accept @index synthetics
                            // but NOT @binance or other exchange-specific synthetics
                            if inst.symbol.contains("@binance") || inst.symbol.contains("@okx") {
                                return false;
                            }
                        }

                        // If group_by specifies instrument_type, synthetic symbol must include the type prefix
                        if group_by.instrument_type {
                            // Synthetics WITH type grouping look like: syn-spot-btc-usd@ or syn-perpetual-btc-usd@
                            let has_type_prefix = inst.symbol.contains("-spot-")
                                || inst.symbol.contains("-perpetual-")
                                || inst.symbol.contains("-future-");
                            if !has_type_prefix {
                                return false;
                            }
                        } else {
                            // If NOT grouping by type, synthetic should NOT have a type prefix
                            // Synthetics without type grouping look like: syn-btc-usd@index
                            let has_type_prefix = inst.symbol.contains("-spot-")
                                || inst.symbol.contains("-perpetual-")
                                || inst.symbol.contains("-future-");
                            if has_type_prefix {
                                return false;
                            }
                        }

                        true
                    })
                    .map(|synthetic| {
                        // Find all real instruments that should aggregate into this synthetic
                        let inputs: Vec<Arc<Instrument>> = real_instruments
                            .iter()
                            .filter(|real_inst| {
                                // Check if this real instrument should be included
                                Self::matches_filter(real_inst, filter)
                                    && Self::should_aggregate_into(real_inst, synthetic, group_by)
                            })
                            .cloned()
                            .collect();

                        InstrumentScope::new(inputs, Arc::clone(synthetic))
                    })
                    .collect()
            }

            // Index: one or more index synthetics, each reads from grouped synthetics
            AggregationType::Index { filter } => {
                synthetic_instruments
                    .iter()
                    .filter(|inst| {
                        // Must be synthetic
                        if !inst.synthetic {
                            return false;
                        }

                        // Must be an index synthetic (not base asset synthetics)
                        // Index synthetics: index_notional_01m@index
                        // Base synthetics: syn-btc-usd@index, syn-spot-btc-usd@index
                        if inst.symbol.starts_with("syn-") {
                            return false;
                        }
                        true
                    })
                    .map(|index_synthetic| {
                        // Index reads from grouped synthetics (syn-*)
                        // Grouped synthetics have venue=Index and instrument_type=Index,
                        // so we only filter on base_asset, quote_asset, and synthetic flag
                        let inputs: Vec<_> = synthetic_instruments
                            .iter()
                            .filter(|syn| {
                                // Must be a grouped synthetic (not an index synthetic)
                                syn.symbol.starts_with("syn-") &&
                                !syn.symbol.starts_with("index") &&
                                // Match base_asset if specified
                                (filter.base_asset.is_empty() || filter.base_asset.contains(&syn.base_asset.symbol)) &&
                                // Match quote_asset if specified
                                (filter.quote_asset.is_empty() || filter.quote_asset.contains(&syn.quote_asset.symbol)) &&
                                // Match synthetic flag if specified
                                filter.synthetic.map_or(true, |s| syn.synthetic == s)
                            })
                            .cloned()
                            .collect();

                        InstrumentScope::new(inputs, Arc::clone(index_synthetic))
                    })
                    .collect()
            }
        }
    }

    /// Check if an instrument matches the filter
    fn matches_filter(inst: &Instrument, filter: &crate::config::InstrumentFilter) -> bool {
        // Empty filter means match all
        if filter.base_asset.is_empty()
            && filter.quote_asset.is_empty()
            && filter.instrument_type.is_empty()
            && filter.venue.is_empty()
            && filter.symbols.is_empty()
            && filter.synthetic.is_none()
        {
            return true;
        }

        // Check base asset
        if !filter.base_asset.is_empty() && !filter.base_asset.contains(&inst.base_asset.symbol) {
            return false;
        }

        // Check quote asset
        if !filter.quote_asset.is_empty() && !filter.quote_asset.contains(&inst.quote_asset.symbol) {
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

        // Check symbols (exact match)
        if !filter.symbols.is_empty() && !filter.symbols.contains(&inst.symbol) {
            return false;
        }

        // Check synthetic flag
        if let Some(synthetic_filter) = filter.synthetic {
            if inst.synthetic != synthetic_filter {
                return false;
            }
        }

        true
    }

    /// Determine if a real instrument should aggregate into a synthetic based on grouping rules
    fn should_aggregate_into(
        real_inst: &Instrument,
        synthetic: &Instrument,
        group_by: &crate::config::GroupBy,
    ) -> bool {
        // Match by base asset (BTC, ETH, etc.)
        if real_inst.base_asset.symbol != synthetic.base_asset.symbol {
            return false;
        }

        // Match by quote asset - real instrument's quote must be in the group_by list
        // Note: The synthetic has the reference currency (USD), but real instruments have USDT, USDC, etc.
        if !group_by.quote_asset.is_empty() && !group_by.quote_asset.contains(&real_inst.quote_asset.symbol) {
            return false;
        }

        // Match by instrument type if group_by.instrument_type is true
        // The synthetic's symbol encodes the instrument type (e.g., "syn-perpetual-btc-usd")
        if group_by.instrument_type {
            let type_str = real_inst.instrument_type.to_string().to_lowercase();
            // Check if the synthetic symbol contains this type
            if !synthetic.symbol.contains(&format!("syn-{}-", type_str)) {
                return false;
            }
        }

        true
    }
}
