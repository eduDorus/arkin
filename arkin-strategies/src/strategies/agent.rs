use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ndarray::{Array, Array3, Ix3};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use rust_decimal::prelude::*;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError, StrategyService};

#[derive(TypedBuilder)]
#[allow(unused)]
pub struct AgentStrategy {
    pubsub: PubSubHandle,
    strategy: Arc<Strategy>,
    model_location: String,
    model_name: String,
    model_version: String,
    action_space: Vec<Decimal>,
    n_layers: usize,
    hidden_size: usize,
    inputs: Vec<FeatureId>,
    #[builder(default)]
    models: RwLock<HashMap<Arc<Instrument>, Arc<Session>>>,
    #[builder(default)]
    current_weight: RwLock<HashMap<Arc<Instrument>, Decimal>>,
    #[builder(default)]
    hidden_states: RwLock<HashMap<Arc<Instrument>, (Array3<f32>, Array3<f32>)>>,
}

#[async_trait]
impl Algorithm for AgentStrategy {
    async fn insight_tick(&self, tick: Arc<InsightsUpdate>) -> Result<(), StrategyError> {
        for instrument in &tick.instruments {
            // Get the model
            // Check if we have a model for the instrument
            // If not, check if we have a model file
            // If we have a model file, load the model
            // If we don't have a model file, log a warning and return None
            if !self.models.read().await.contains_key(instrument) {
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
                self.models.write().await.insert(instrument.clone(), model.into());
            }
            let model = self.models.read().await.get(instrument).expect("Model not found").clone();

            // Lock the current weight map
            let current_weight = self
                .current_weight
                .read()
                .await
                .get(instrument)
                .cloned()
                .unwrap_or(Decimal::ZERO);

            // Get or initialize hidden and cell states
            let hidden_states = self.hidden_states.read().await;
            let (hidden, cell) = hidden_states.get(instrument).cloned().unwrap_or_else(|| {
                (
                    Array3::zeros((self.n_layers, 1, self.hidden_size)),
                    Array3::zeros((self.n_layers, 1, self.hidden_size)),
                )
            });
            drop(hidden_states); // Release the read lock

            // Prepare the observation from tick insights (single step)
            let mut obs: Vec<f32> = self
                .inputs
                .iter()
                .map(|feature_id| {
                    tick.insights
                        .iter()
                        .find(|i| {
                            i.instrument == Some(instrument.clone())
                                && i.feature_id == *feature_id
                                && (i.insight_type == InsightType::Normalized
                                    || i.insight_type == InsightType::Prediction)
                        })
                        .map(|i| i.value)
                        .unwrap_or(0.0) // Default to 0.0 if feature missing
                })
                .map(|v| v as f32)
                .collect();
            if obs.len() != self.inputs.len() {
                warn!("Incomplete observation data for instrument {}", instrument);
                continue; // Skip this instrument if data is missing
            }
            obs.push(current_weight.to_f32().unwrap()); // Append current weight to observation
            let obs_array = Array::from_shape_vec((1, self.inputs.len() + 1), obs).expect("Failed to create obs array");

            // Convert inputs to ort::Value
            let obs_value = Tensor::from_array(obs_array).expect("Failed to create obs value");
            let hidden_value = Tensor::from_array(hidden).expect("Failed to create hidden value");
            let cell_value = Tensor::from_array(cell).expect("Failed to create cell value");

            // Print all inputs

            // Run the model with correct input names
            let outputs = model
                .run(
                    ort::inputs![
                        "obs" => obs_value,
                        "lstm_hidden_in" => hidden_value,
                        "lstm_cell_in" => cell_value
                    ]
                    .unwrap(),
                )
                .expect("Failed to run model");

            // Extract outputs (assuming order: action, lstm_hidden_out, lstm_cell_out)
            let action = outputs[0]
                .try_extract_tensor::<i64>()
                .expect("Failed to extract action")
                .to_owned();
            let new_hidden = outputs[1]
                .try_extract_tensor::<f32>()
                .expect("Failed to extract hidden")
                .to_owned();
            let new_cell = outputs[2]
                .try_extract_tensor::<f32>()
                .expect("Failed to extract cell")
                .to_owned();

            // Convert states to Array3
            let new_hidden = new_hidden.into_dimensionality::<Ix3>().expect("Hidden state is not 3D");
            let new_cell = new_cell.into_dimensionality::<Ix3>().expect("Cell state is not 3D");

            // Update hidden states
            self.hidden_states
                .write()
                .await
                .insert(instrument.clone(), (new_hidden, new_cell));

            // info!("Action: {:?}", action[0]);
            let weight = self.action_space[action[0] as usize];
            // info!("Weight: {}", weight);

            // update the last action
            self.current_weight.write().await.insert(instrument.clone(), weight);
            let signal = Signal::builder()
                .event_time(tick.event_time)
                .strategy(self.strategy.clone())
                .instrument(instrument.clone())
                .weight(weight)
                .build();
            self.pubsub.publish(signal).await;
        }

        Ok(())
    }
}

#[async_trait]
impl StrategyService for AgentStrategy {}

#[async_trait]
impl RunnableService for AgentStrategy {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting Agent Strategy...");

        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::InsightsUpdate(tick) => {
                            debug!("Agent Strategy received insight tick: {}", tick.event_time);
                            self.insight_tick(tick).await?;
                        }
                        Event::Finished => {
                          break;
                      }
                        _ => {}
                    }
                    self.pubsub.ack().await;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        info!("Agent Strategy stopped.");
        Ok(())
    }
}
