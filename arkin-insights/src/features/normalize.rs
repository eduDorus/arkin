use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use serde::Deserialize;
use statrs::distribution::{ContinuousCDF, Normal};
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{math::interp, Feature, FeatureStore};

#[derive(Debug, Clone)]
pub struct RobustScaler {
    feature_data: HashMap<(Uuid, FeatureId), (f64, f64)>,
}

impl RobustScaler {
    pub fn new(data_location: &str, version: &str) -> Self {
        let mut feature_data = HashMap::new();

        let quantiles = Self::load(data_location, version);
        for q in quantiles.data.into_iter() {
            feature_data.insert((q.instrument_id, q.feature_id.into()), (q.median, q.iqr));
        }
        RobustScaler { feature_data }
    }

    fn load(data_location: &str, version: &str) -> Quantiles {
        let file = std::fs::File::open(format!("{data_location}/{version}.json")).expect("Failed to open file");
        let quantiles: Quantiles = serde_json::from_reader(file).expect("Failed to parse JSON");
        quantiles
    }

    #[allow(dead_code)]
    pub fn features(&self) -> HashSet<FeatureId> {
        self.feature_data.keys().map(|(_, f)| f.clone()).collect()
    }

    pub fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self
            .feature_data
            .get(&key)
            .unwrap_or_else(|| panic!("Feature ID not found: {:?}", key));
        (x - median) / iqr
    }

    pub fn transform_normal(&self, x: f64) -> f64 {
        x / 1.3489795003921636
    }

    #[allow(dead_code)]
    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self
            .feature_data
            .get(&key)
            .unwrap_or_else(|| panic!("Feature ID not found: {:?}", key));
        x * iqr + median
    }

    #[allow(dead_code)]
    pub fn inverse_transform_normal(&self, x: f64) -> f64 {
        x * 1.3489795003921636
    }
}

#[allow(dead_code)]
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
    /// Create a new transformer with a specified number of quantiles and output distribution
    pub fn new(data_location: &str, version: &str, output_distribution: DistributionType) -> Self {
        let quantiles = Self::load(data_location, version);
        let references = quantiles.levels;
        let feature_quantiles = quantiles
            .data
            .into_iter()
            .map(|q| ((q.instrument_id, q.feature_id.into()), q.quantiles))
            .collect();
        QuantileTransformer {
            feature_quantiles,
            references,
            output_distribution,
        }
    }

    fn load(data_location: &str, version: &str) -> Quantiles {
        let file = std::fs::File::open(format!("{data_location}/{version}.json")).expect("Failed to open file");
        let quantile_data: Quantiles = serde_json::from_reader(file).expect("Failed to parse JSON");
        quantile_data
    }

    #[allow(dead_code)]
    pub fn features(&self) -> HashSet<FeatureId> {
        self.feature_quantiles.keys().map(|(_, f)| f.clone()).collect()
    }

    /// Transform a value x for a given feature_id
    pub fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        if x.is_nan() {
            warn!("NaN value detected in transform");
            return x;
        }
        let key = (instrument_id, feature_id.clone());
        let quantiles = if let Some(quantiles) = self.feature_quantiles.get(&key) {
            quantiles
        } else {
            warn!("Feature ID: {} not found in quantile transformer", feature_id);
            return x;
        };

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

    #[allow(dead_code)]
    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, y: f64) -> f64 {
        // Step 1: Get quantiles and references for the feature
        let key = (instrument_id, feature_id.clone());
        let quantiles = self
            .feature_quantiles
            .get(&key)
            .unwrap_or_else(|| panic!("Feature ID not found: {:?}", key));

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

#[derive(Debug, Display, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NormalizeFeatureType {
    Quantile,
    Robust,
    QuantileRobust,
}

#[derive(Debug, TypedBuilder)]
pub struct NormalizeFeature {
    transformer: QuantileTransformer,
    scaler: RobustScaler,
    input: Vec<FeatureId>,
    output: FeatureId,
    method: NormalizeFeatureType,
    persist: bool,
}

#[async_trait]
impl Feature for NormalizeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        self.input.clone()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        state: &FeatureStore,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        debug!("Robust scaling...");

        //  Get data
        let insights = self
            .input
            .iter()
            .filter_map(|id| {
                // Get the value
                let value = state.last(instrument, id, event_time)?;

                // Check if value is nan
                if value.is_nan() {
                    warn!("NaN value detected in normalization");
                    return None;
                }

                let altered_value = match self.method {
                    NormalizeFeatureType::Quantile => self.transformer.transform(instrument.id, id, value),
                    NormalizeFeatureType::Robust => self.scaler.transform(instrument.id, id, value),
                    NormalizeFeatureType::QuantileRobust => {
                        let transformed_value = self.transformer.transform(instrument.id, id, value);
                        self.scaler.transform_normal(transformed_value)
                    }
                };

                // Create Insight
                Some(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(Some(pipeline.clone()))
                        .instrument(instrument.clone())
                        .feature_id(id.clone())
                        .value(altered_value)
                        .insight_type(InsightType::Normalized)
                        .persist(self.persist)
                        .build(),
                )
            })
            .map(Arc::new)
            .collect::<Vec<_>>();

        // Save insights to state
        state.insert_batch(insights.as_slice());

        Some(insights)
    }

    // async fn async_calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
    //     self.calculate(instrument, timestamp)
    // }
}
