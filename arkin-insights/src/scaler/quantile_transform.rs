use std::{collections::HashMap, sync::Arc};

use statrs::distribution::{ContinuousCDF, Normal};
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{math::interp, state::InsightsState, Feature};

use super::QuantileData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionType {
    Uniform,
    Normal,
}

#[derive(Debug, Clone)]
pub struct QuantileTransformer {
    feature_quantiles: HashMap<(Uuid, FeatureId), Vec<f64>>, // Quantiles per feature_id
    references: Vec<f64>,                                    // Shared probability levels (e.g., 0 to 1)
    output_distribution: DistributionType,                   // "uniform" or "normal"
}

impl QuantileTransformer {
    /// Load a transformer from a JSON file
    pub fn load(file_path: &str, output_distribution: DistributionType) -> Self {
        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let scaler_data: QuantileData = serde_json::from_reader(file).expect("Failed to parse JSON");
        QuantileTransformer::new(scaler_data, output_distribution)
    }

    pub fn features(&self) -> Vec<FeatureId> {
        self.feature_quantiles.keys().map(|(_, f)| f.clone()).collect()
    }

    /// Create a new transformer with a specified number of quantiles and output distribution
    fn new(scaler_data: QuantileData, output_distribution: DistributionType) -> Self {
        let references = scaler_data.levels;
        let feature_quantiles = scaler_data
            .data
            .into_iter()
            .map(|q| ((q.instrument_id, q.feature_id.into()), q.quantiles))
            .collect();
        QuantileTransformer {
            feature_quantiles: feature_quantiles,
            references,
            output_distribution: output_distribution,
        }
    }

    /// Transform a value x for a given feature_id
    fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let quantiles = self.feature_quantiles.get(&key).expect("Feature ID not found in quantiles");
        // Forward interpolation
        let forward = interp(x, quantiles, &self.references);

        // Reverse interpolation
        let quantiles_rev: Vec<f64> = quantiles.iter().rev().map(|&v| -v).collect();
        let references_rev: Vec<f64> = self.references.iter().rev().map(|&v| -v).collect();
        let reverse = interp(-x, &quantiles_rev, &references_rev);

        // Average the two interpolations
        let p = 0.5 * (forward - reverse);

        // Apply the output distribution
        match self.output_distribution {
            DistributionType::Uniform => p,
            DistributionType::Normal => {
                let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
                let clip_min = normal.inverse_cdf(1e-7); // Avoid -infinity
                let clip_max = normal.inverse_cdf(1.0 - 1e-7); // Avoid +infinity
                normal.inverse_cdf(p).max(clip_min).min(clip_max)
            }
        }
    }

    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, y: f64) -> f64 {
        // Step 1: Get quantiles and references for the feature
        let key = (instrument_id, feature_id.clone());
        let quantiles = self.feature_quantiles.get(&key).expect("Feature ID not found");

        // Step 2: Compute p based on output distribution
        let p = match self.output_distribution {
            DistributionType::Uniform => y,
            DistributionType::Normal => {
                let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
                normal.cdf(y)
            }
        };

        // Step 3: Interpolate p to get x
        interp(p, &self.references, quantiles)
    }
}

#[derive(Debug, TypedBuilder)]
pub struct QuantileTransformerFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    transformer: QuantileTransformer,
    input: Vec<FeatureId>,
    output: FeatureId,
    persist: bool,
}

impl Feature for QuantileTransformerFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        self.input.clone()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Robust scaling...");

        //  Get data
        let insights = self
            .transformer
            .features()
            .iter()
            .filter_map(|id| {
                // Get the value
                let value = self.insight_state.last(Some(instrument.clone()), id.clone(), event_time)?;

                // Calculate scaled values
                let transformed_value = self.transformer.transform(instrument.id, id, value);

                // Create Insight
                Some(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(Some(self.pipeline.clone()))
                        .instrument(Some(instrument.clone()))
                        .feature_id(id.clone())
                        .value(transformed_value)
                        .insight_type(InsightType::Transformed)
                        .persist(self.persist)
                        .build()
                        .into(),
                )
            })
            .collect::<Vec<_>>();

        Some(insights)
    }
}
