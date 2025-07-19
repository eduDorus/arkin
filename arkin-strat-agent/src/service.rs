use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tokio::sync::Mutex;
use tonic::{codec::CompressionEncoding, transport::Channel};
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

const BATCH_SIZE: usize = 1;
const SEQUENCE_LENGTH: usize = 192;
const NUM_FEATURES_OBS: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5
const NUM_FEATURES_STATE: usize = 3;
const NUM_MASK: usize = 3;
const POSSIBLE_WEIGHTS: [f32; 3] = [0., -1., 1.];

const SHAPE_0: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_OBS as i64];
const SHAPE_1: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_STATE as i64];
const SHAPE_2: [i64; 2] = [BATCH_SIZE as i64, NUM_MASK as i64];

#[derive(TypedBuilder)]
pub struct AgentStrategy {
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    strategy: Arc<Strategy>,
    client: Mutex<GrpcInferenceServiceClient<Channel>>,
    returns_feature_id: FeatureId,
    input_features_ids: Vec<FeatureId>,
    input_features: DashMap<FeatureId, VecDeque<f32>>,
    weight: Mutex<VecDeque<f32>>,
    trade_cum_pnl: Mutex<VecDeque<f32>>,
    trade_max_cum: Mutex<f32>,
    trade_dd: Mutex<VecDeque<f32>>,
}

impl AgentStrategy {
    pub fn new(time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>, strategy: Arc<Strategy>) -> Arc<Self> {
        let url = "http://192.168.100.100:8001";
        let channel = Channel::from_static(url).connect_lazy();
        let client = GrpcInferenceServiceClient::new(channel).send_compressed(CompressionEncoding::Gzip); // Marginally slower with compression (but we will save some bandwith)

        let returns_feature_id = "price_percent_change_01min".to_owned().into();
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

        // In AgentStrategy::new
        let input_features = DashMap::new();
        for feature_id in &input_features_ids {
            input_features.insert(feature_id.clone(), VecDeque::<f32>::with_capacity(SEQUENCE_LENGTH));
        }
        Self {
            identifier: "strat_agent".to_owned(),
            time,
            publisher,
            strategy,
            client: Mutex::new(client),
            returns_feature_id,
            input_features_ids: input_features_ids,
            input_features,
            weight: Mutex::new(VecDeque::with_capacity(SEQUENCE_LENGTH)),
            trade_cum_pnl: Mutex::new(VecDeque::with_capacity(SEQUENCE_LENGTH)),
            trade_max_cum: Mutex::new(0.0),
            trade_dd: Mutex::new(VecDeque::with_capacity(SEQUENCE_LENGTH)),
        }
        .into()
    }

    fn create_mask(&self, new_weight: f32, possible_weights: &[f32]) -> Vec<bool> {
        if new_weight < 0.0 {
            possible_weights.iter().map(|&w| w >= new_weight && w <= 0.0).collect()
        } else if new_weight > 0.0 {
            possible_weights.iter().map(|&w| w <= new_weight && w >= 0.0).collect()
        } else {
            vec![true; possible_weights.len()]
        }
    }

    async fn calculate_trade_cum_pnl(&self, current_return: f32) {
        let weight_guard = self.weight.lock().await;
        let mut pnl_guard = self.trade_cum_pnl.lock().await;
        let mut max_cum_guard = self.trade_max_cum.lock().await;

        let prev_weight = weight_guard.back().copied().unwrap_or(0.0);
        let net_pnl = prev_weight * current_return;

        if prev_weight == 0.0 {
            // Enter/exit trade: reset
            pnl_guard.push_back(net_pnl);
            *max_cum_guard = net_pnl.max(0.0); // Init max to current if positive
        } else {
            let last_cum_pnl = pnl_guard.back().copied().unwrap_or(0.0);
            let cum_pnl = last_cum_pnl + net_pnl;
            pnl_guard.push_back(cum_pnl);
            *max_cum_guard = (*max_cum_guard).max(cum_pnl);
        }
        if pnl_guard.len() > SEQUENCE_LENGTH {
            pnl_guard.pop_front();
        }
    }

    async fn calculate_trade_dd(&self) {
        let pnl_guard = self.trade_cum_pnl.lock().await;
        let max_cum = *self.trade_max_cum.lock().await;
        let mut dd_guard = self.trade_dd.lock().await;

        let cum_pnl = pnl_guard.back().copied().unwrap_or(0.0);
        let dd = ((max_cum - cum_pnl) / (max_cum + 1e-6)).max(0.0).min(1.0);
        dd_guard.push_back(dd);
        if dd_guard.len() > SEQUENCE_LENGTH {
            dd_guard.pop_front();
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn insight_tick(&self, update: &InsightsUpdate) {
        // first fill the data from the update
        for insight in update
            .insights
            .iter()
            .filter(|i| self.input_features_ids.contains(&i.feature_id))
        {
            if let Some(mut deque) = self.input_features.get_mut(&insight.feature_id) {
                deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                if deque.len() > SEQUENCE_LENGTH {
                    deque.pop_front();
                }
            }
        }

        // For returns (assume single value; error if multiple/none)
        let current_return = update
            .insights
            .iter()
            .find(|i| i.feature_id == self.returns_feature_id)
            .map(|v| v.value as f32)
            .unwrap_or(0.0);
        self.calculate_trade_cum_pnl(current_return).await;
        self.calculate_trade_dd().await;

        // Create Feature Input
        let mut input_data_flat_0: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS);
        for _batch in 0..BATCH_SIZE {
            // For now, single batch
            for seq in 0..SEQUENCE_LENGTH {
                for feat_idx in 0..NUM_FEATURES_OBS {
                    let feature_id = &self.input_features_ids[feat_idx];
                    if let Some(deque) = self.input_features.get(feature_id) {
                        let len = deque.len();
                        let val = if seq >= SEQUENCE_LENGTH - len {
                            deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0)
                        } else {
                            0.0 // Pad front
                        };
                        input_data_flat_0.push(val);
                    } else {
                        input_data_flat_0.push(0.0);
                    }
                }
            }
        }

        // Create State Input
        let mut input_data_flat_1: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_STATE);
        let state_deques = [
            self.weight.lock().await.clone(), // Clone for read; optimize if perf issue
            self.trade_cum_pnl.lock().await.clone(),
            self.trade_dd.lock().await.clone(),
        ];
        for _batch in 0..BATCH_SIZE {
            for seq in 0..SEQUENCE_LENGTH {
                for state_idx in 0..NUM_FEATURES_STATE {
                    let deque = &state_deques[state_idx];
                    let len = deque.len();
                    let val = if seq >= SEQUENCE_LENGTH - len {
                        deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0)
                    } else {
                        0.0 // Pad front
                    };
                    input_data_flat_1.push(val);
                }
            }
        }

        // Create Mask
        let current_weight = self.weight.lock().await.back().copied().unwrap_or(0.0);
        let mask = self.create_mask(current_weight, &POSSIBLE_WEIGHTS);
        let input_data_flat_2: Vec<bool> = (0..BATCH_SIZE).flat_map(|_| mask.clone()).collect();

        // Construct the inference request
        let inputs = vec![
            InferInputTensor {
                name: "OBSERVATION".to_string(),
                shape: SHAPE_0.to_vec(),
                datatype: "FP32".to_string(),
                contents: Some(InferTensorContents {
                    fp32_contents: input_data_flat_0,
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            },
            InferInputTensor {
                name: "STATE".to_string(),
                shape: SHAPE_1.to_vec(),
                datatype: "FP32".to_string(),
                contents: Some(InferTensorContents {
                    fp32_contents: input_data_flat_1,
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            },
            InferInputTensor {
                name: "MASK".to_string(),
                shape: SHAPE_2.to_vec(),
                datatype: "BOOL".to_string(),
                contents: Some(InferTensorContents {
                    bool_contents: input_data_flat_2,
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            },
        ];

        let outputs = vec![
            InferRequestedOutputTensor {
                name: "ACTION".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "ACTION_SPACE".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "WEIGHT".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "PROBABILITY".to_string(),
                parameters: std::collections::HashMap::new(),
            },
        ];

        // Show the input
        // info!(target: "strat::agent", "AGENT INPUT: {:?}", inputs);

        // Step 3: Send the request and handle the response
        let infer_request = ModelInferRequest {
            model_name: "agent".to_string(),
            model_version: "".to_string(),
            id: "".to_string(),
            parameters: std::collections::HashMap::new(),
            inputs,
            outputs,
            raw_input_contents: vec![],
        };

        let response = {
            let mut client = self.client.lock().await;
            match client.model_infer(infer_request.clone()).await {
                Ok(resp) => resp.into_inner(),
                Err(e) => {
                    warn!("Inference failed: {}", e);
                    return;
                }
            }
        };

        let outputs = response.outputs;
        if outputs.is_empty() {
            warn!(target: "strat::agent", "No outputs in response");
            return;
        }

        // After getting response and checking !outputs.is_empty()

        let mut outputs_map: HashMap<String, InferOutputTensor> = HashMap::new();
        for output in &outputs {
            outputs_map.insert(output.name.clone(), output.clone());
        }

        if let Some(weight_out) = outputs_map.get("WEIGHT") {
            // Validate datatype and shape (assume FP32, shape [1] for single f32 weight)
            if weight_out.datatype != "FP32" {
                warn!(target: "strat::agent","Unexpected datatype for WEIGHT: {}", weight_out.datatype);
                return;
            }
            if weight_out.shape != vec![1i64] {
                warn!(target: "strat::agent","Unexpected shape for WEIGHT: {:?}", weight_out.shape);
                return;
            }

            let new_weight: f32;
            if !response.raw_output_contents.is_empty() {
                // Raw mode: Assume outputs ordered, find index
                let index = outputs.iter().position(|o| o.name == "WEIGHT").unwrap();
                let raw_bytes = &response.raw_output_contents[index];
                if raw_bytes.len() != 4 {
                    warn!(target: "strat::agent","Invalid raw bytes length for f32: {}", raw_bytes.len());
                    return;
                }
                new_weight = f32::from_le_bytes([raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]]);
            } else if let Some(contents) = &weight_out.contents {
                if contents.fp32_contents.len() != 1 {
                    warn!(target: "strat::agent","Invalid fp32 contents length: {}", contents.fp32_contents.len());
                    return;
                }
                new_weight = contents.fp32_contents[0];
            } else {
                warn!(target: "strat::agent","No contents or raw for WEIGHT");
                return;
            }

            // Now use new_weight: Push to deque
            let mut weight_guard = self.weight.lock().await;
            weight_guard.push_back(new_weight);
            if weight_guard.len() > SEQUENCE_LENGTH {
                weight_guard.pop_front();
            }
            let prev_weight = if weight_guard.len() >= 2 {
                *weight_guard.get(weight_guard.len() - 2).unwrap_or(&0.0)
            } else {
                0.0
            };
            drop(weight_guard); // Unlock early

            let allocation_change = new_weight - prev_weight;
            if allocation_change != 0.0 {
                let instrument = update.instruments[0].clone(); // Assume InsightsUpdate exposes instrument
                let quantity = Decimal::from_f32_retain(allocation_change.abs()).unwrap_or(Decimal::ZERO);
                if quantity.is_zero() {
                    return; // Skip trivial
                }
                let order = ExecutionOrder::builder()
                    .id(Uuid::new_v4())
                    .strategy(Some(self.strategy.clone()))
                    .instrument(instrument)
                    .exec_strategy_type(ExecutionStrategyType::Taker)
                    .side(if allocation_change > 0.0 {
                        MarketSide::Buy
                    } else {
                        MarketSide::Sell
                    })
                    .set_price(Decimal::ZERO)
                    .set_quantity(quantity)
                    .status(ExecutionOrderStatus::New)
                    .created(self.time.now().await)
                    .updated(self.time.now().await)
                    .build();
                self.publisher
                    .publish(Event::NewTakerExecutionOrder(order.clone().into()))
                    .await;
                info!(target: "strat::agent", "send {} execution order for {} quantity {}", order.side, order.instrument, order.quantity);
            }
        } else {
            warn!(target: "strat::agent","Missing WEIGHT output");
        }
    }
}

#[async_trait]
impl Runnable for AgentStrategy {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::InsightsUpdate(vo) => self.insight_tick(vo).await,
            e => warn!(target: "strat::agent", "received unused event {}", e),
        }
    }
}
