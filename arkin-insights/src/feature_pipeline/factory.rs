use std::sync::Arc;

use arkin_core::{Instrument, PersistenceReader};
use tracing::info;

use crate::{
    config::{FeatureConfig, GroupBy, InstrumentSelector, PipelineConfig},
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
        let mut instrument_query =
            SyntheticGenerator::build_instrument_query(&pipeline_config.global_instrument_selector);
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
                    assert_eq!(c.input.len(), c.method.len(), "input and method must have the same length");

                    // Build scopes using the new two-step approach
                    let filtered = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector,
                    );
                    let scopes = Self::build_scopes_from_filtered(
                        &filtered,
                        real_instruments,
                        synthetic_instruments,
                        &c.group_by,
                    );

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            LagFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .lag(c.lag[i])
                                .method(c.method[i])
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
                    assert_eq!(c.input.len(), c.method.len(), "input and method must have the same length");

                    // Build scopes using the new two-step approach: filter + group by
                    let filtered = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector,
                    );
                    let scopes = Self::build_scopes_from_filtered(
                        &filtered,
                        real_instruments,
                        synthetic_instruments,
                        &c.group_by,
                    );
                    tracing::info!(
                        "Range feature with selector {:?} and group_by {:?} has {} scopes",
                        c.instrument_selector,
                        c.group_by,
                        scopes.len()
                    );

                    for i in 0..c.input.len() {
                        let input_feature = persistence.get_feature_id(&c.input[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            RangeFeature::builder()
                                .input(input_feature)
                                .output(output_feature)
                                .data(c.data[i])
                                .method(c.method[i])
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

                    // Build scopes using the new two-step approach
                    let filtered = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector,
                    );
                    let scopes = Self::build_scopes_from_filtered(
                        &filtered,
                        real_instruments,
                        synthetic_instruments,
                        &c.group_by,
                    );

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            DualRangeFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .data(c.data[i])
                                .method(c.method[i])
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

                    // Build scopes using the new two-step approach for both selectors
                    let filtered_1 = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector_1,
                    );
                    let filtered_2 = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector_2,
                    );

                    // Merge the two filtered sets for scopes
                    let mut merged = filtered_1;
                    merged.extend(filtered_2);

                    let scopes =
                        Self::build_scopes_from_filtered(&merged, real_instruments, synthetic_instruments, &c.group_by);

                    for i in 0..c.input_1.len() {
                        let input_1_feature = persistence.get_feature_id(&c.input_1[i]).await;
                        let input_2_feature = persistence.get_feature_id(&c.input_2[i]).await;
                        let output_feature = persistence.get_feature_id(&c.output[i]).await;

                        features.push(Arc::new(
                            TwoValueFeature::builder()
                                .input_1(input_1_feature)
                                .input_2(input_2_feature)
                                .output(output_feature)
                                .method(c.method[i])
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

                    // Build scopes using the new two-step approach
                    let filtered = Self::filter_instruments_by_selector(
                        real_instruments,
                        synthetic_instruments,
                        &c.instrument_selector,
                    );
                    let scopes = Self::build_scopes_from_filtered(
                        &filtered,
                        real_instruments,
                        synthetic_instruments,
                        &c.group_by,
                    );

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

    /// Filter instruments based on the selector
    /// Returns the filtered set of instruments that match the selector criteria
    fn filter_instruments_by_selector(
        real_instruments: &[Arc<Instrument>],
        synthetic_instruments: &[Arc<Instrument>],
        selector: &InstrumentSelector,
    ) -> Vec<Arc<Instrument>> {
        let mut result = Vec::new();

        // Check synthetic flag to determine which set to use
        if selector.synthetic == Some(true) {
            result.extend(
                synthetic_instruments
                    .iter()
                    .filter(|inst| Self::matches_filter(inst, selector))
                    .cloned(),
            );
        } else if selector.synthetic == Some(false) {
            result.extend(
                real_instruments
                    .iter()
                    .filter(|inst| Self::matches_filter(inst, selector))
                    .cloned(),
            );
        } else {
            // No synthetic filter - include both real and synthetic
            result.extend(
                real_instruments
                    .iter()
                    .filter(|inst| Self::matches_filter(inst, selector))
                    .cloned(),
            );
            result.extend(
                synthetic_instruments
                    .iter()
                    .filter(|inst| Self::matches_filter(inst, selector))
                    .cloned(),
            );
        }

        result
    }

    /// Build scopes from filtered instruments based on GroupBy configuration
    /// Each scope represents a collection of instruments that should be processed together
    fn build_scopes_from_filtered(
        filtered_instruments: &[Arc<Instrument>],
        _real_instruments: &[Arc<Instrument>],
        _synthetic_instruments: &[Arc<Instrument>],
        group_by: &GroupBy,
    ) -> Vec<InstrumentScope> {
        if filtered_instruments.is_empty() {
            return vec![];
        }

        // If GroupBy is empty (all flags false/empty), create 1:1 scopes
        if !group_by.base_asset && group_by.quote_asset.is_empty() && !group_by.instrument_type && !group_by.venue {
            return filtered_instruments
                .iter()
                .map(|inst| InstrumentScope::single(Arc::clone(inst)))
                .collect();
        }

        // Otherwise, group instruments according to GroupBy config
        // For real instruments being grouped, create synthetic outputs
        // For synthetic instruments, create 1:1 scopes
        let mut result = Vec::new();

        for inst in filtered_instruments {
            if inst.synthetic {
                // Synthetic instruments get 1:1 scopes
                result.push(InstrumentScope::single(Arc::clone(inst)));
            } else {
                // Real instruments would be grouped by the group_by config
                // For now, keep as 1:1 until we implement grouping logic
                result.push(InstrumentScope::single(Arc::clone(inst)));
            }
        }

        result
    }

    /// Check if an instrument matches the filter
    fn matches_filter(inst: &Instrument, filter: &InstrumentSelector) -> bool {
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
    fn should_aggregate_into(real_inst: &Instrument, synthetic: &Instrument, group_by: &GroupBy) -> bool {
        // Match by base asset (BTC, ETH, etc.)
        if real_inst.base_asset.symbol != synthetic.base_asset.symbol {
            return false;
        }

        // Match by quote asset - real instrument's quote must be in the group_by list
        // Note: The synthetic has the reference currency (USD), but real instruments have USDT, USDC, etc.
        if !group_by.quote_asset.is_empty() && !group_by.quote_asset.contains(&real_inst.quote_asset.symbol) {
            return false;
        }

        // Match by venue if group_by.venue is true
        if group_by.venue && real_inst.venue.name != synthetic.venue.name {
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
