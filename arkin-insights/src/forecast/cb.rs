use std::{fmt, sync::Arc};

use anyhow::Result;
use catboost_rs::Model;
use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{state::InsightsState, Computation};

#[derive(TypedBuilder)]
pub struct CatBoostFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    model_location: String,
    model_name: String,
    model_version: String,
    #[builder(default)]
    models: DashMap<Arc<Instrument>, Arc<Model>>,
    input_numerical: Vec<FeatureId>,
    input_categorical: Vec<FeatureId>,
    output: FeatureId,
}

impl fmt::Debug for CatBoostFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CatBoostFeature")
            .field("pipeline", &self.pipeline)
            .field("input_numerical", &self.input_numerical)
            .field("input_categorical", &self.input_categorical)
            .field("output", &self.output)
            .finish()
    }
}

impl Computation for CatBoostFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        self.input_numerical
            .iter()
            .chain(self.input_categorical.iter())
            .cloned()
            .collect()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating Log Returns...");

        // Retrieve the values for the feature over the window period
        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get the model
                // Check if we have a model for the instrument
                // If not, check if we have a model file
                // If we have a model file, load the model
                // If we don't have a model file, log a warning and return None
                if !self.models.contains_key(instrument) {
                    let filename = format!(
                        "{}/{}_{}_{}.cbm",
                        self.model_location, instrument.secondary_id, self.model_name, self.model_version
                    );

                    let model = Model::load(&filename).expect("Failed to load model");
                    self.models.insert(instrument.clone(), Arc::new(model));
                }
                let model = self.models.get(instrument).expect("Model not found").value().clone();

                //  Get data
                let numerical_data = self
                    .input_numerical
                    .iter()
                    .filter_map(|id| {
                        let value = self.insight_state.last(Some(instrument.clone()), id.clone(), event_time)?;
                        value.to_f32()
                    })
                    .collect::<Vec<_>>();

                let mut categorical_data = self
                    .input_categorical
                    .iter()
                    .filter_map(|id| {
                        let value = self.insight_state.last(Some(instrument.clone()), id.clone(), event_time)?;
                        let int_value = value.to_i64()?;
                        Some(int_value.to_string())
                    })
                    .collect::<Vec<_>>();

                // Validate that we have all the data
                if numerical_data.len() != self.input_numerical.len()
                    || categorical_data.len() != self.input_categorical.len()
                {
                    warn!("Not enough data to calculate forecast");
                    return None;
                }

                // Add the instrument ID
                categorical_data.push(instrument.secondary_id.to_string());

                // Apply the model
                let prediction = model
                    .calc_model_prediction(vec![numerical_data], vec![categorical_data])
                    .expect("Failed to calculate model prediction");

                // info!("Prediction: {:?}", prediction);
                // None
                // Convert to decimal
                let prediction = Decimal::from_f64(prediction[0]).expect("Failed to convert prediction to decimal");

                // Return insight
                Some(
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(prediction)
                        .build()
                        .into(),
                )
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
