use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ndarray::{Array, Array3, Ix3};
use ort::{session::Session, value::Tensor};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError, StrategyService};

#[derive(Debug, Clone, TypedBuilder)]
struct AgentState {
    #[builder(default)]
    current_weight: Decimal,
    #[builder(default)]
    delta_weight: Decimal,
    #[builder(default)]
    trade_pnl: Decimal,
}

impl AgentState {
    pub fn update_pnl(&mut self, pct_change: Decimal) {
        self.trade_pnl += self.current_weight * pct_change;
    }

    pub fn update_state(&mut self, proposed_weight: Decimal, commission_rate: Decimal) {
        let mut actual_new_weight = self.current_weight; // Default to no change

        if self.current_weight == Decimal::ZERO {
            // Not in a trade
            if proposed_weight != Decimal::ZERO {
                self.entry_signal_count += 1;
                info!("Increasing entry signal count to {}", self.entry_signal_count);
                if self.entry_signal_count >= 2 {
                    // Require at least 2 consecutive signals
                    actual_new_weight = proposed_weight; // Enter the trade
                    self.entry_signal_count = 0; // Reset counter after entry
                }
            } else {
                self.entry_signal_count = 0; // Reset if sequence is interrupted
            }
        } else {
            // In a trade
            if proposed_weight == Decimal::ZERO {
                self.exit_signal_count += 1;
                info!("Increasing exit signal count to {}", self.exit_signal_count);
                if self.exit_signal_count >= 2 {
                    // Require at least 2 consecutive signals
                    actual_new_weight = Decimal::ZERO; // Exit the trade
                    self.exit_signal_count = 0; // Reset counter after exit
                }
            } else {
                self.exit_signal_count = 0; // Reset if sequence is interrupted
                if proposed_weight != self.current_weight {
                    actual_new_weight = proposed_weight; // Adjust position immediately
                }
            }
        }

        // Update the state based on actual_new_weight
        let current_weight = self.current_weight;
        let new_weight = actual_new_weight;

        if current_weight == Decimal::ZERO && new_weight != Decimal::ZERO {
            // Entering a trade
            let delta_position = new_weight.abs();
            let commission = commission_rate * delta_position;
            self.trade_pnl -= commission;
            self.holding_steps = Decimal::ONE;
        } else if current_weight != Decimal::ZERO && new_weight == Decimal::ZERO {
            // Exiting a trade
            self.holding_steps = Decimal::ZERO;
            self.trade_pnl = Decimal::ZERO;
        } else if current_weight != Decimal::ZERO && new_weight != Decimal::ZERO && current_weight != new_weight {
            // Adjusting position
            self.holding_steps += Decimal::ONE;
        } else if current_weight != Decimal::ZERO && new_weight == current_weight {
            // Staying in a trade
            self.holding_steps += Decimal::ONE;
        } else if current_weight == Decimal::ZERO && new_weight == Decimal::ZERO {
            // Staying out of a trade
            self.holding_steps = Decimal::ZERO;
            self.trade_pnl = Decimal::ZERO;
        } else {
            warn!(
                "Unexpected state: current_weight: {}, new_weight: {}",
                current_weight, new_weight
            );
        }

        // Update the current weight
        self.current_weight = new_weight;
    }

    pub fn set_hidden(&mut self, hidden: Array3<f32>) {
        self.hidden = hidden;
    }

    pub fn set_cell(&mut self, cell: Array3<f32>) {
        self.cell = cell;
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentState {{ current_weight: {}, holding_steps: {}, trade_pnl: {}, hidden: {:?}, cell: {:?} }}",
            self.current_weight, self.holding_steps, self.trade_pnl, self.hidden, self.cell
        )
    }
}

#[derive(TypedBuilder)]
#[allow(unused)]
pub struct AgentStrategy {
    pubsub: PubSubHandle,
    strategy: Arc<Strategy>,
    model_location: String,
    model_name: String,
    model_version: String,
    sequence_length: usize,
    inputs: Vec<FeatureId>,
    input_change: FeatureId,
    commission_rate: Decimal,
    #[builder(default)]
    models: RwLock<HashMap<Arc<Instrument>, Arc<Session>>>,
    #[builder(default)]
    agent_state: RwLock<HashMap<Arc<Instrument>, AgentState>>,
}

#[async_trait]
impl Algorithm for AgentStrategy {
    async fn insight_tick(&self, tick: Arc<InsightsUpdate>) -> Result<(), StrategyError> {
        for instrument in &tick.instruments {
            // Get or init  the model
            if !self.models.read().await.contains_key(instrument) {
                info!("Initializing model for {}", instrument);
                let filename = format!(
                    "{}/{}_{}_{}.onnx",
                    self.model_location, self.model_name, self.model_version, instrument.id
                );

                let model = Session::builder()
                    .expect("Failed to create session builder")
                    // .with_optimization_level(GraphOptimizationLevel::Level3)
                    // .expect("Failed to set optimization level")
                    // .with_intra_threads(4)
                    // .expect("Failed to set intra threads")
                    .with_deterministic_compute(true)
                    .expect("Failed to set deterministic compute")
                    .commit_from_file(filename)
                    .expect("Failed to commit from file");
                self.models.write().await.insert(instrument.clone(), model.into());
            }
            let model = self.models.read().await.get(instrument).expect("Model not found").clone();

            // Prepare the observation from tick insights (single step)
            let mut obs: Vec<f64> = self
                .inputs
                .iter()
                .filter_map(|feature_id| {
                    tick.insights
                        .iter()
                        .find(|i| {
                            i.instrument == Some(instrument.clone())
                                && i.feature_id == *feature_id
                                && (i.insight_type == InsightType::Normalized
                                    || i.insight_type == InsightType::Prediction)
                        })
                        .map(|i| i.value)
                })
                .map(|v| v)
                .collect();
            if obs.len() != self.inputs.len() {
                warn!("Incomplete observation data for instrument {}", instrument);
                continue; // Skip this instrument if data is missing
            }

            // Get the current change
            let pct_change = tick
                .insights
                .iter()
                .find(|i| {
                    i.instrument == Some(instrument.clone())
                        && i.feature_id == self.input_change
                        && i.insight_type == InsightType::Continuous
                })
                .map(|i| i.value)
                .unwrap_or(0.0);

            // Lock the current weight map
            let mut agent_state = self
                .agent_state
                .read()
                .await
                .get(instrument)
                .cloned()
                .unwrap_or(AgentState::new(self.n_layers, self.hidden_size));
            agent_state.update_pnl(Decimal::from_f64(pct_change).unwrap());

            // Add agent state
            obs.push(agent_state.current_weight.to_f64().unwrap_or(0.0));
            obs.push((agent_state.holding_steps / dec!(10)).to_f64().unwrap());
            obs.push((agent_state.trade_pnl * dec!(100)).to_f64().unwrap());

            let obs = obs
                .into_iter()
                .map(|v| ((v * 1_000_000.0).round() / 1_000_000.0) as f32)
                .collect::<Vec<_>>();

            // Here we have all the observations
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("observations_8.txt")
                .expect("Failed to open observations.txt");
            let row = format!(
                "{}, {}, {}",
                tick.event_time,
                instrument.id,
                obs.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", ")
            );
            writeln!(file, "{}", row).expect("Failed to write observation row");

            // Prepare the hidden and cell states
            let obs_array = Array::from_shape_vec((1, self.inputs.len() + 3), obs).expect("Failed to create obs array");

            // Convert inputs to ort::Value
            let obs_value = Tensor::from_array(obs_array).expect("Failed to create obs value");
            let hidden_value = Tensor::from_array(agent_state.hidden.clone()).expect("Failed to create hidden value");
            let cell_value = Tensor::from_array(agent_state.cell.clone()).expect("Failed to create cell value");

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
            let new_weight = self.action_space[action[0] as usize];
            let new_hidden = new_hidden.into_dimensionality::<Ix3>().expect("Hidden state is not 3D");
            let new_cell = new_cell.into_dimensionality::<Ix3>().expect("Cell state is not 3D");

            // Update the agent state
            agent_state.update_state(new_weight, self.commission_rate);
            agent_state.set_hidden(new_hidden);
            agent_state.set_cell(new_cell);
            self.agent_state.write().await.insert(instrument.clone(), agent_state.clone());
            info!("Agent state for {}: {}", instrument, agent_state);

            let signal = Signal::builder()
                .event_time(tick.event_time)
                .strategy(self.strategy.clone())
                .instrument(instrument.clone())
                .weight(new_weight)
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
