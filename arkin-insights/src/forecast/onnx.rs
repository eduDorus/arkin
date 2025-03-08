use std::sync::Arc;

use dashmap::DashMap;
use ndarray::Array;
use ort::session::{builder::GraphOptimizationLevel, Session};
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

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
                self.model_location, instrument.id, self.model_name, self.model_version
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
                self.insight_state
                    .intervals(Some(instrument.clone()), id.clone(), event_time, self.sequence_length)
            })
            .collect::<Vec<Vec<_>>>();

        // Validate that we have all the data
        if data.iter().any(|d| d.len() != self.sequence_length) {
            warn!("Not enough data to calculate forecast");
            return None;
        }

        // let input_array = Array::from_elem((1, SEQ_LEN, INPUTS_LEN), 1.0f32);
        let input_array = Array::from_shape_fn((1, self.sequence_length, self.input.len()), |(i, j, k)| data[k][i + j]);

        // Apply the model
        let input = ort::value::Tensor::from_array(input_array.clone().into_dyn()).expect("Failed to create tensor");
        let outputs = model.run(ort::inputs!["input" => input].unwrap()).expect("Failed to run model");
        if let Some(predictions) = outputs["output"].try_extract_tensor::<f32>().ok() {
            println!("ORT Predictions: {:?}", predictions.as_slice());
        } else {
            warn!("Failed to extract predictions");
        }

        // // Return insight
        // let insight = Insight::builder()
        //     .event_time(event_time)
        //     .pipeline(Some(self.pipeline.clone()))
        //     .instrument(Some(instrument.clone()))
        //     .feature_id(self.output.clone())
        //     .value(prediction)
        //     .insight_type(InsightType::Prediction)
        //     .persist(self.persist)
        //     .build()
        //     .into();

        // Some(vec![insights])
        None
    }
}
