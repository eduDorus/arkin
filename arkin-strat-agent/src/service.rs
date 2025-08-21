use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tracing::{info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{allocation::AllocationEngine, infer_http::HTTPInferencer, utils::should_infer};

const BATCH_SIZE: usize = 1;
const SEQUENCE_LENGTH: usize = 192;
const NUM_FEATURES_OBS: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5
const NUM_STATE_OBS: usize = 1;
const POSSIBLE_WEIGHTS: [f32; 3] = [-1.0, 0.0, 1.0];

const SHAPE_0: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_OBS as i64];
const SHAPE_1: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_STATE_OBS as i64];

const INPUT_NAMES: [&str; 2] = ["INPUT__0", "INPUT__1"];
const OUTPUT_NAMES: [&str; 4] = ["OUTPUT__0", "OUTPUT__1", "OUTPUT__2", "OUTPUT__3"];
const OUTPUT_WEIGHT_NAME: &str = "OUTPUT__2";

struct AgentState {
    input_features: HashMap<FeatureId, VecDeque<f32>>,
    input_state: HashMap<FeatureId, VecDeque<f32>>,
}

#[derive(TypedBuilder)]
pub struct AgentStrategy {
    #[allow(unused)]
    strategy: Arc<Strategy>,
    interval: u64,
    allocation: AllocationEngine,
    client: HTTPInferencer, // or GRPCInferencer
    input_features_ids: Vec<FeatureId>,
    input_state_ids: Vec<FeatureId>,
    agent_inst_states: DashMap<Arc<Instrument>, AgentState>,
}

impl AgentStrategy {
    pub fn new(strategy: Arc<Strategy>, capital_per_inst: Decimal, interval: u64) -> Arc<Self> {
        let url = "http://192.168.100.100:8000/v2/models/agent/infer";
        let client = HTTPInferencer::new(url.to_owned());

        // Input features
        let input_features_ids: Vec<FeatureId> = vec![
            "price_percent_change_10min".to_owned().into(),
            "price_percent_change_15min".to_owned().into(),
            "price_percent_change_30min".to_owned().into(),
            "price_percent_change_60min".to_owned().into(),
            "price_imbalance_10min".to_owned().into(),
            "price_imbalance_15min".to_owned().into(),
            "price_imbalance_30min".to_owned().into(),
            "price_imbalance_60min".to_owned().into(),
            "price_relative_position_10min".to_owned().into(),
            "price_relative_position_15min".to_owned().into(),
            "price_relative_position_30min".to_owned().into(),
            "price_relative_position_60min".to_owned().into(),
            "price_relative_range_10min".to_owned().into(),
            "price_relative_range_15min".to_owned().into(),
            "price_relative_range_30min".to_owned().into(),
            "price_relative_range_60min".to_owned().into(),
            "price_acceleration_10min".to_owned().into(),
            "price_acceleration_15min".to_owned().into(),
            "price_acceleration_30min".to_owned().into(),
            "price_acceleration_60min".to_owned().into(),
            "price_volume_covariance_10min".to_owned().into(),
            "price_volume_covariance_15min".to_owned().into(),
            "price_volume_covariance_30min".to_owned().into(),
            "price_volume_covariance_60min".to_owned().into(),
            "volatility_10min".to_owned().into(),
            "volatility_15min".to_owned().into(),
            "volatility_30min".to_owned().into(),
            "volatility_60min".to_owned().into(),
            "volume_percent_change_10min".to_owned().into(),
            "volume_percent_change_15min".to_owned().into(),
            "volume_percent_change_30min".to_owned().into(),
            "volume_percent_change_60min".to_owned().into(),
            "volume_relative_position_10min".to_owned().into(),
            "volume_relative_position_15min".to_owned().into(),
            "volume_relative_position_30min".to_owned().into(),
            "volume_relative_position_60min".to_owned().into(),
            "volume_relative_range_10min".to_owned().into(),
            "volume_relative_range_15min".to_owned().into(),
            "volume_relative_range_30min".to_owned().into(),
            "volume_relative_range_60min".to_owned().into(),
        ];
        // let input_features = HashMap::new();
        // for feature_id in &input_features_ids {
        //     input_features.insert(feature_id.clone(), VecDeque::<f32>::with_capacity(SEQUENCE_LENGTH));
        // }

        // State features
        let input_state_ids: Vec<FeatureId> = vec!["weight".to_owned().into()];
        // let input_state = HashMap::new();
        // for id in &input_state_ids {
        //     input_state.insert(id.clone(), VecDeque::<f32>::with_capacity(SEQUENCE_LENGTH));
        // }
        Self {
            strategy: strategy.to_owned(),
            interval,
            allocation: AllocationEngine::new(capital_per_inst, strategy.to_owned()),
            client,
            input_features_ids: input_features_ids,
            input_state_ids,
            agent_inst_states: DashMap::new(),
        }
        .into()
    }

    async fn warmup_insight_tick(&self, _ctx: Arc<CoreCtx>, update: &InsightsUpdate) {
        if !should_infer(update.event_time, self.interval) {
            warn!(target: "strat::agent", "insights event skipped due to interval misalignment, update at {}", update.event_time);
            return;
        }

        // Initialize input state for new instruments
        for inst in &update.instruments {
            if !self.agent_inst_states.contains_key(inst) {
                let input_features: HashMap<FeatureId, VecDeque<f32>> = self
                    .input_features_ids
                    .iter()
                    .map(|id| (id.clone(), VecDeque::with_capacity(SEQUENCE_LENGTH)))
                    .collect();
                let input_state: HashMap<FeatureId, VecDeque<f32>> = self
                    .input_state_ids
                    .iter()
                    .map(|id| (id.clone(), VecDeque::with_capacity(SEQUENCE_LENGTH)))
                    .collect();
                self.agent_inst_states.insert(
                    inst.to_owned(),
                    AgentState {
                        input_features,
                        input_state,
                    },
                );
            }
        }

        // Add new features
        for inst in &update.instruments {
            if let Some(mut state) = self.agent_inst_states.get_mut(inst) {
                for insight in update.insights.iter().filter(|i| {
                    self.input_features_ids.contains(&i.feature_id)
                        && i.insight_type == InsightType::Normalized
                        && i.instrument == Some(inst.clone())
                }) {
                    if let Some(deque) = state.input_features.get_mut(&insight.feature_id) {
                        deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                        if deque.len() > SEQUENCE_LENGTH {
                            deque.pop_front();
                        }
                    }
                }
            } else {
                warn!(target: "strat::agent", "Missing input state for instrument {}", inst);
            }
        }
    }

    async fn insight_tick(&self, ctx: Arc<CoreCtx>, update: &InsightsUpdate) {
        if !should_infer(update.event_time, self.interval) {
            warn!(target: "strat::agent", "insights event skipped due to interval misalignment, update at {}", update.event_time);
            return;
        }

        // Add new features
        for inst in &update.instruments {
            if let Some(mut state) = self.agent_inst_states.get_mut(inst) {
                for insight in update.insights.iter().filter(|i| {
                    self.input_features_ids.contains(&i.feature_id)
                        && i.insight_type == InsightType::Normalized
                        && i.instrument == Some(inst.clone())
                }) {
                    if let Some(deque) = state.input_features.get_mut(&insight.feature_id) {
                        deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                        if deque.len() > SEQUENCE_LENGTH {
                            deque.pop_front();
                        }
                    }
                }

                // Check if history full; skip if not (avoids zero-padding obs)
                if state.input_features.iter().any(|(_k, v)| v.len() < SEQUENCE_LENGTH) {
                    info!(target: "strat::agent", "Skipping inference: insufficient history");
                    continue;
                }

                // Create Feature Input
                let mut input_data_0: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS);
                for _batch in 0..BATCH_SIZE {
                    for seq in 0..SEQUENCE_LENGTH {
                        for feat_idx in 0..NUM_FEATURES_OBS {
                            let feature_id = &self.input_features_ids[feat_idx];
                            if let Some(deque) = state.input_features.get(feature_id) {
                                let len = deque.len();
                                let val = if seq >= SEQUENCE_LENGTH - len {
                                    deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0)
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
                let mut input_data_1: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_STATE_OBS);
                let max_weight = POSSIBLE_WEIGHTS.iter().fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
                let scale_factor = 1.0 / (max_weight * 1.3489795003921636);
                for _batch in 0..BATCH_SIZE {
                    for seq in 0..SEQUENCE_LENGTH {
                        for state_idx in 0..NUM_STATE_OBS {
                            let state_id = &self.input_state_ids[state_idx];
                            if let Some(deque) = state.input_state.get(state_id) {
                                let len = deque.len();
                                let val = if seq >= SEQUENCE_LENGTH - len {
                                    let weight = deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0);
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
                        &INPUT_NAMES,
                        &[&input_data_0, &input_data_1],
                        &[&SHAPE_0, &SHAPE_1],
                        &OUTPUT_NAMES,
                    )
                    .await;

                if let Some(outputs) = response {
                    let new_weight = outputs.get(OUTPUT_WEIGHT_NAME).cloned().unwrap_or_default()[0];
                    info!(target: "strat::agent", "New weight is {}", new_weight);

                    // Now use new_weight: Push to deque
                    if let Some(deque) = state.input_state.get_mut(&FeatureId::from("weight".to_owned())) {
                        deque.push_back(new_weight.to_f32().unwrap_or(0.0)); // Convert Decimal to f32, default 0 on error
                        if deque.len() > SEQUENCE_LENGTH {
                            deque.pop_front();
                        }
                        let time = ctx.now().await;
                        if let Some(order) = self.allocation.update(time, &update.instruments[0], new_weight) {
                            ctx.publish(Event::NewTakerExecutionOrder(order.clone().into())).await;
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
