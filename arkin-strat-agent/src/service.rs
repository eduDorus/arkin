use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    allocation::AllocationEngine, config::ModelSettings, infer_grpc::GrpcInferencer, infer_http::HTTPInferencer,
    traits::InferenceService, utils::should_infer,
};

// const BATCH_SIZE: usize = 1;
// const SEQUENCE_LENGTH: usize = 192;
// const NUM_FEATURES_OBS: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5
// const NUM_STATE_OBS: usize = 1;
// const POSSIBLE_WEIGHTS: [f32; 3] = [-1.0, 0.0, 1.0];

// const SHAPE_0: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_OBS as i64];
// const SHAPE_1: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_STATE_OBS as i64];

// const INPUT_NAMES: [&str; 2] = ["INPUT__0", "INPUT__1"];
// const OUTPUT_NAMES: [&str; 4] = ["OUTPUT__0", "OUTPUT__1", "OUTPUT__2", "OUTPUT__3"];
// const OUTPUT_WEIGHT_NAME: &str = "OUTPUT__2";

struct AgentState {
    input_features: HashMap<FeatureId, VecDeque<f32>>,
    input_state: HashMap<FeatureId, VecDeque<f32>>,
}

#[derive(TypedBuilder)]
pub struct AgentStrategy {
    #[allow(unused)]
    strategy: Arc<Strategy>,
    allocation: AllocationEngine,
    client: Arc<dyn InferenceService>,
    inference_interval: u64,
    input_features_ids: Vec<FeatureId>,
    input_state_ids: Vec<FeatureId>,
    agent_inst_states: DashMap<Arc<Instrument>, AgentState>,
    model_settings: ModelSettings,
}

impl AgentStrategy {
    pub fn new(
        strategy: Arc<Strategy>,
        capital_per_inst: Decimal,
        inference_interval: u64,
        input_feature_ids: Vec<FeatureId>,
        input_state_ids: Vec<FeatureId>,
        inference_host: String,
        inference_port: u16,
        inference_type: String,
        model_settings: ModelSettings,
    ) -> Arc<Self> {
        let client = match inference_type.as_str() {
            "http" => {
                let base_url = format!("http://{}:{}", inference_host, inference_port);
                Arc::new(HTTPInferencer::new(base_url)) as Arc<dyn InferenceService>
            }
            "grpc" => {
                let url = format!("http://{}:{}", inference_host, inference_port);
                Arc::new(GrpcInferencer::new(&url)) as Arc<dyn InferenceService>
            }
            _ => {
                panic!("Unsupported inference type: {}", inference_type);
            }
        };

        Self {
            strategy: strategy.to_owned(),
            allocation: AllocationEngine::new(capital_per_inst, strategy.to_owned()),
            client,
            inference_interval,
            input_features_ids: input_feature_ids,
            input_state_ids,
            agent_inst_states: DashMap::new(),
            model_settings,
        }
        .into()
    }

    async fn warmup_insight_tick(&self, _ctx: Arc<CoreCtx>, update: &InsightsUpdate) {
        if !should_infer(update.event_time, self.inference_interval) {
            debug!(target: "strat::agent", "insights event skipped due to interval misalignment, update at {}", update.event_time);
            return;
        }

        let sequence_length = self.model_settings.sequence_length;

        // Initialize input state for new instruments
        let instruments = update.instruments();
        for inst in &instruments {
            if !self.agent_inst_states.contains_key(inst) {
                let input_features: HashMap<FeatureId, VecDeque<f32>> = self
                    .input_features_ids
                    .iter()
                    .map(|id| (id.clone(), VecDeque::with_capacity(sequence_length)))
                    .collect();
                let input_state: HashMap<FeatureId, VecDeque<f32>> = self
                    .input_state_ids
                    .iter()
                    .map(|id| (id.clone(), VecDeque::with_capacity(sequence_length)))
                    .collect();
                self.agent_inst_states.insert(
                    inst.to_owned(),
                    AgentState {
                        input_features,
                        input_state,
                    },
                );
                info!(target: "strat::agent", "Initialized state for instrument {}", inst);
            }
        }

        // Add new features
        for inst in &instruments {
            if let Some(mut state) = self.agent_inst_states.get_mut(inst) {
                for insight in update.insights.iter().filter(|i| {
                    self.input_features_ids.contains(&i.feature_id)
                        && i.insight_type == InsightType::Normalized
                        && i.instrument == inst.clone()
                }) {
                    if let Some(deque) = state.input_features.get_mut(&insight.feature_id) {
                        debug!(target: "strat::agent", "Adding feature {} with value {} for instrument {}", insight.feature_id, insight.value, inst);
                        deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                        if deque.len() > sequence_length {
                            deque.pop_front();
                        }
                    } else {
                        warn!(target: "strat::agent", "missing feature deque for feature {}", insight.feature_id);
                    }
                }
            } else {
                warn!(target: "strat::agent", "Missing input state for instrument {}", inst);
            }
        }
    }

    async fn insight_tick(&self, ctx: Arc<CoreCtx>, update: &InsightsUpdate) {
        if !should_infer(update.event_time, self.inference_interval) {
            debug!(target: "strat::agent", "insights event skipped due to interval misalignment, update at {}", update.event_time);
            return;
        }

        let batch_size = self.model_settings.batch_size;
        let sequence_length = self.model_settings.sequence_length;
        let num_features_obs = self.model_settings.num_features_obs;
        let num_state_obs = self.model_settings.num_state_obs;
        let possible_weights = &self.model_settings.possible_weights;
        let input_names: Vec<&str> = self.model_settings.input_names.iter().map(String::as_str).collect();
        let output_names: Vec<&str> = self.model_settings.output_names.iter().map(String::as_str).collect();
        let shape_0 = [batch_size as i64, sequence_length as i64, num_features_obs as i64];
        let shape_1 = [batch_size as i64, sequence_length as i64, num_state_obs as i64];
        let output_weight_name = &self.model_settings.output_weight_name;
        let model_name_prefix = &self.model_settings.model_name_prefix;
        let model_name_postfix = &self.model_settings.model_name_postfix;

        // Add new features
        let instruments = update.instruments();
        for inst in &instruments {
            if let Some(mut state) = self.agent_inst_states.get_mut(inst) {
                for insight in update.insights.iter().filter(|i| {
                    self.input_features_ids.contains(&i.feature_id)
                        && i.insight_type == InsightType::Normalized
                        && i.instrument == inst.clone()
                }) {
                    if let Some(deque) = state.input_features.get_mut(&insight.feature_id) {
                        debug!(target: "strat::agent", "Adding feature {} with value {} for instrument {}", insight.feature_id, insight.value, inst);
                        deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                        if deque.len() > sequence_length {
                            deque.pop_front();
                        }
                    } else {
                        warn!(target: "strat::agent", "missing feature deque for feature {}", insight.feature_id);
                    }
                }

                // Check if history full; skip if not (avoids zero-padding obs)
                // Find the length of each feature:
                let min_length = state.input_features.values().map(|v| v.len()).min().unwrap_or(0);
                if min_length < sequence_length {
                    warn!(target: "strat::agent", "Skipping inference: insufficient history with {} entries", min_length);
                    continue;
                }

                // Create Feature Input
                let mut input_data_0: Vec<f32> = Vec::with_capacity(batch_size * sequence_length * num_features_obs);
                for _batch in 0..batch_size {
                    for seq in 0..sequence_length {
                        for feat_idx in 0..num_features_obs {
                            let feature_id = &self.input_features_ids[feat_idx];
                            if let Some(deque) = state.input_features.get(feature_id) {
                                let len = deque.len();
                                let val = if seq >= sequence_length - len {
                                    deque.get(seq - (sequence_length - len)).copied().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                input_data_0.push(val);
                            } else {
                                input_data_0.push(0.0);
                            }
                        }
                    }
                }

                // Create State Input
                let mut input_data_1: Vec<f32> = Vec::with_capacity(batch_size * sequence_length * num_state_obs);
                let max_weight = possible_weights.iter().fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
                let scale_factor = 1.0 / (max_weight * 1.348_979_5);
                for _batch in 0..batch_size {
                    for seq in 0..sequence_length {
                        for state_idx in 0..num_state_obs {
                            let state_id = &self.input_state_ids[state_idx];
                            if let Some(deque) = state.input_state.get(state_id) {
                                let len = deque.len();
                                let val = if seq >= sequence_length - len {
                                    let weight = deque.get(seq - (sequence_length - len)).copied().unwrap_or(0.0);
                                    weight * scale_factor
                                } else {
                                    0.0
                                };
                                input_data_1.push(val);
                            } else {
                                input_data_1.push(0.0);
                            }
                        }
                    }
                }

                // Prepare inputs for inference
                let response = self
                    .client
                    .request(
                        &format!(
                            "{}_{}_{}",
                            model_name_prefix,
                            inst.venue_symbol.to_lowercase(),
                            model_name_postfix
                        ),
                        &input_names,
                        &[&input_data_0, &input_data_1],
                        &[&shape_0, &shape_1],
                        &output_names,
                    )
                    .await;

                if let Some(outputs) = response {
                    let new_weight = outputs.get(output_weight_name).cloned().unwrap_or_default()[0];
                    info!(target: "strat::agent", "New weight is {}", new_weight);

                    // Now use new_weight: Push to deque
                    if let Some(deque) = state.input_state.get_mut(&FeatureId::from("weight".to_owned())) {
                        deque.push_back(new_weight.to_f32().unwrap_or(0.0)); // Convert Decimal to f32, default 0 on error
                        if deque.len() > sequence_length {
                            deque.pop_front();
                        }
                        let time = ctx.now().await;
                        if let Some(order) = self.allocation.update(time, inst, new_weight) {
                            ctx.publish(Event::NewExecutionOrder(order.clone().into())).await;
                            info!(target: "strat::agent", "send {} execution order for {} quantity {}", order.side, order.instrument, order.quantity);
                        }
                    } else {
                        warn!(target: "strat::agent", "missing weight deque");
                        return;
                    }
                } else {
                    warn!(target: "strat::agent", "failed to get a response from inference server");
                }
            } else {
                warn!(target: "strat::agent", "missing input state for instrument {}", inst);
            }
        }
    }

    async fn tick_update(&self, update: Arc<Tick>) {
        self.allocation.update_price(update);
    }
}

#[async_trait]
impl Runnable for AgentStrategy {
    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        match &event {
            Event::WarmupInsightsUpdate(vo) => self.warmup_insight_tick(ctx, vo).await,
            Event::InsightsUpdate(vo) => self.insight_tick(ctx, vo).await,
            Event::TickUpdate(t) => self.tick_update(t.clone()).await,
            e => warn!(target: "strat::agent", "received unused event {}", e),
        }
    }
}
