use std::{collections::HashMap, sync::Arc};

use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{state::InsightsState, Feature};

use super::QuantileData;

#[derive(Debug, Clone)]
pub struct RobustScaler {
    feature_data: HashMap<(Uuid, FeatureId), (f64, f64)>,
}

impl RobustScaler {
    pub fn load(file_path: &str) -> Self {
        let file = std::fs::File::open(file_path).expect("Failed to open file");
        let scaler_data: QuantileData = serde_json::from_reader(file).expect("Failed to parse JSON");
        RobustScaler::new(scaler_data)
    }

    pub fn features(&self) -> Vec<FeatureId> {
        self.feature_data.keys().map(|(_, f)| f.clone()).collect()
    }

    pub fn new(scaler_data: QuantileData) -> Self {
        let feature_data = scaler_data
            .data
            .into_iter()
            .map(|q| {
                let key = (q.instrument_id, q.feature_id.into());
                let value = (q.median, q.iqr);
                (key, value)
            })
            .collect();
        RobustScaler { feature_data }
    }

    pub fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self.feature_data.get(&key).expect("Feature ID not found");
        (x - median) / iqr
    }

    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self.feature_data.get(&key).expect("Feature ID not found");
        x * iqr + median
    }
}

#[derive(Debug, TypedBuilder)]
pub struct RobustScalerFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    scaler: RobustScaler,
    input: Vec<FeatureId>,
    output: FeatureId,
    persist: bool,
}

impl Feature for RobustScalerFeature {
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
            .scaler
            .features()
            .iter()
            .filter_map(|id| {
                // Get the value
                let value = self.insight_state.last(Some(instrument.clone()), id.clone(), event_time)?;

                // Calculate scaled values
                let scaled_value = self.scaler.transform(instrument.id, id, value);

                // Create Insight
                Some(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(Some(self.pipeline.clone()))
                        .instrument(Some(instrument.clone()))
                        .feature_id(id.clone())
                        .value(scaled_value)
                        .insight_type(InsightType::Scaled)
                        .persist(self.persist)
                        .build()
                        .into(),
                )
            })
            .collect::<Vec<_>>();

        Some(insights)
    }
}
