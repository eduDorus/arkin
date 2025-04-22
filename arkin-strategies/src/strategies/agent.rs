use std::fmt;
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ndarray::{Array, Array3, Ix3};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
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
    hidden: Array3<f32>,
    cell: Array3<f32>,
    current_weight: Decimal,
    holding_steps: Decimal,
    trade_pnl: Decimal,
}

impl AgentState {
    pub fn new(n_layers: usize, hidden_size: usize) -> Self {
        Self {
            hidden: Array3::ones((n_layers, 1, hidden_size)),
            cell: Array3::ones((n_layers, 1, hidden_size)),
            current_weight: Decimal::ZERO,
            holding_steps: Decimal::ZERO,
            trade_pnl: Decimal::ZERO,
        }
    }

    pub fn update_pnl(&mut self, pct_change: Decimal) {
        self.trade_pnl += self.current_weight * pct_change;
    }

    pub fn update_state(&mut self, new_weight: Decimal, commission_rate: Decimal) {
        let current_weight = self.current_weight;
        let delta_position = (new_weight - current_weight).abs();
        let commission = commission_rate * delta_position;

        // Entering a trade
        if current_weight == Decimal::ZERO && new_weight != Decimal::ZERO {
            self.holding_steps = Decimal::ONE;
            self.trade_pnl -= commission;

        // Closing a trade
        } else if current_weight != Decimal::ZERO && new_weight == Decimal::ZERO {
            self.holding_steps = Decimal::ZERO;
            self.trade_pnl = Decimal::ZERO;
        } else if current_weight != Decimal::ZERO && new_weight != Decimal::ZERO && current_weight != new_weight {
            // Changing position
            self.holding_steps += Decimal::ONE;
        }
        // Staying in a trade
        else if current_weight != Decimal::ZERO && new_weight != Decimal::ZERO && current_weight == new_weight {
            self.holding_steps += Decimal::ONE;
        }
        // Staying out of a trade (current_weight == Decimal::ZERO && new_weight == Decimal::ZERO)
        else if current_weight == Decimal::ZERO && new_weight == Decimal::ZERO {
            self.holding_steps = Decimal::ZERO;
            self.trade_pnl = Decimal::ZERO;
        } else {
            warn!(
                "Unexpected state: current_weight: {}, new_weight: {}",
                current_weight, new_weight
            );
        }

        // Update current weight to the new weight
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
    action_space: Vec<Decimal>,
    n_layers: usize,
    hidden_size: usize,
    inputs: Vec<FeatureId>,
    input_change: FeatureId,
    commission_rate: Decimal,
    #[builder(default)]
    models: RwLock<HashMap<Arc<Instrument>, Arc<Session>>>,
    #[builder(default)]
    agent_state: RwLock<HashMap<Arc<Instrument>, AgentState>>,
    #[builder(default)]
    hidden_states: RwLock<HashMap<Arc<Instrument>, (Array3<f32>, Array3<f32>)>>,
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
                    .with_optimization_level(GraphOptimizationLevel::Level3)
                    .expect("Failed to set optimization level")
                    .with_intra_threads(4)
                    .expect("Failed to set intra threads")
                    .commit_from_file(filename)
                    .expect("Failed to commit from file");
                self.models.write().await.insert(instrument.clone(), model.into());
            }
            let model = self.models.read().await.get(instrument).expect("Model not found").clone();

            // Prepare the observation from tick insights (single step)
            let mut obs: Vec<f32> = self
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
                .map(|v| v as f32)
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
            obs.push(agent_state.current_weight.to_f32().unwrap_or(0.0));
            obs.push((agent_state.holding_steps / dec!(10)).to_f32().unwrap());
            obs.push((agent_state.trade_pnl * dec!(100)).to_f32().unwrap());

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
