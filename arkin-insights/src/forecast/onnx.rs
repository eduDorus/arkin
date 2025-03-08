use std::sync::Arc;

use dashmap::DashMap;
use ndarray::Array;
use ort::session::{builder::GraphOptimizationLevel, Session};
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    features::{QuantileTransformer, RobustScaler},
    state::InsightsState,
    Feature,
};

#[derive(Debug, TypedBuilder)]
pub struct OnnxFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    model_location: String,
    model_name: String,
    model_version: String,
    #[builder(default)]
    models: DashMap<Arc<Instrument>, Arc<Session>>,
    input: Vec<FeatureId>,
    output: FeatureId,
    sequence_length: usize,
    output_feature: FeatureId,
    quantile_transformer: QuantileTransformer,
    robust_scaler: RobustScaler,
    persist: bool,
}

impl Feature for OnnxFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        self.input.clone()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating forecast...");

        // Get the model
        // Check if we have a model for the instrument
        // If not, check if we have a model file
        // If we have a model file, load the model
        // If we don't have a model file, log a warning and return None
        if !self.models.contains_key(instrument) {
            info!("Initializing model for {}", instrument);
            let filename = format!(
                "{}/{}_{}_{}.onnx",
                self.model_location, self.model_name, self.model_version, instrument.id
            );

            let model = Session::builder()
                .expect("Failed to create session builder")
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .expect("Failed to set optimization level")
                .with_intra_threads(4)
                .expect("Failed to set intra threads")
                .commit_from_file(filename)
                .expect("Failed to commit from file");
            self.models.insert(instrument.clone(), model.into());
        }
        let model = self.models.get(instrument).expect("Model not found").value().clone();

        //  Get data
        let data = self
            .input
            .iter()
            .map(|id| {
                let values = self.insight_state.intervals(
                    Some(instrument.clone()),
                    id.clone(),
                    event_time,
                    self.sequence_length,
                );
                // info!("Values for {}: {:?}", id, values);
                let transformed_values = values
                    .into_iter()
                    .map(|v| self.quantile_transformer.transform(instrument.id, id, v))
                    .collect::<Vec<_>>();
                // info!("Transformed Values for {}: {:?}", id, transformed_values);
                let scaled_values = transformed_values
                    .into_iter()
                    .map(|v| self.robust_scaler.transform_normal(v))
                    .collect::<Vec<_>>();
                // info!("Scaled Values for {}: {:?}", id, scaled_values);
                scaled_values
            })
            .collect::<Vec<Vec<_>>>();

        // Validate that we have all the data
        if data.iter().any(|d| d.len() != self.sequence_length) {
            warn!("Not enough data to calculate forecast");
            return None;
        }

        // let input_array = Array::from_elem((1, SEQ_LEN, INPUTS_LEN), 1.0f32);
        let input_array =
            Array::from_shape_fn((1, self.sequence_length, self.input.len()), |(i, j, k)| data[k][i + j] as f32);
        // info!("Input Array: {:?}", input_array);

        // Apply the model
        let input = ort::value::Tensor::from_array(input_array.clone().into_dyn()).expect("Failed to create tensor");
        let outputs = model.run(ort::inputs!["input" => input].unwrap()).expect("Failed to run model");
        if let Some(predictions) = outputs["output"].try_extract_tensor::<f32>().ok() {
            let predication = predictions.as_slice().unwrap();
            info!("ORT Predictions: {:?}", predication);
            // Inverse scale the predictions
            let scaled_prediction = self
                .robust_scaler
                .inverse_transform_normal(predication.first().unwrap().clone() as f64);
            info!("Inverse Scaled Prediction: {:?}", scaled_prediction);
            // Inverse transform the predictions
            let prediction =
                self.quantile_transformer
                    .inverse_transform(instrument.id, &self.output_feature, scaled_prediction);
            info!("Inverse Transformed Prediction: {}", prediction);

            // // Return insight
            let insight = Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .value(prediction)
                .insight_type(InsightType::Prediction)
                .persist(self.persist)
                .build()
                .into();

            Some(vec![insight])
        } else {
            warn!("Failed to extract predictions");
            return None;
        }
    }
}
