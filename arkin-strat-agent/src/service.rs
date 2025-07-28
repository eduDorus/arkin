use std::{
    collections::{HashMap, VecDeque},
    fs::OpenOptions,
    io::{BufWriter, Seek, SeekFrom},
    sync::Arc,
};

use async_trait::async_trait;
use csv::Writer;
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
const NUM_STATE_OBS: usize = 1;
const NUM_MASK: usize = 4;
const POSSIBLE_WEIGHTS: [f32; 4] = [0.0, 1.0, 2.0, 3.0];

const SHAPE_0: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_OBS as i64];
const SHAPE_1: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_STATE_OBS as i64];
const SHAPE_2: [i64; 2] = [BATCH_SIZE as i64, NUM_MASK as i64];

#[derive(TypedBuilder)]
pub struct AgentStrategy {
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    strategy: Arc<Strategy>,
    client: Mutex<GrpcInferenceServiceClient<Channel>>,
    input_features_ids: Vec<FeatureId>,
    input_features: DashMap<FeatureId, VecDeque<f32>>,
    input_state_ids: Vec<FeatureId>,
    input_state: DashMap<FeatureId, VecDeque<f32>>,
}

impl AgentStrategy {
    pub fn new(time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>, strategy: Arc<Strategy>) -> Arc<Self> {
        let url = "http://192.168.100.100:8001";
        let channel = Channel::from_static(url).connect_lazy();
        let client = GrpcInferenceServiceClient::new(channel).send_compressed(CompressionEncoding::Gzip); // Marginally slower with compression (but we will save some bandwith)

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
        let input_features = DashMap::new();
        for feature_id in &input_features_ids {
            input_features.insert(feature_id.clone(), VecDeque::<f32>::with_capacity(SEQUENCE_LENGTH));
        }

        // State features
        let input_state_ids: Vec<FeatureId> = vec!["weight".to_owned().into()];
        let input_state = DashMap::new();
        for id in &input_state_ids {
            input_state.insert(id.clone(), VecDeque::<f32>::with_capacity(SEQUENCE_LENGTH));
        }
        Self {
            identifier: "strat_agent".to_owned(),
            time,
            publisher,
            strategy,
            client: Mutex::new(client),
            input_features_ids: input_features_ids,
            input_features,
            input_state_ids,
            input_state,
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

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn insight_tick(&self, update: &InsightsUpdate) {
        let mut new_features = HashMap::new();
        for insight in update
            .insights
            .iter()
            .filter(|i| self.input_features_ids.contains(&i.feature_id) && i.insight_type == InsightType::Normalized)
        {
            if let Some(mut deque) = self.input_features.get_mut(&insight.feature_id) {
                new_features.insert(insight.feature_id.clone(), insight.value as f32);
                deque.push_back(insight.value as f32); // Convert Decimal to f32, default 0 on error
                if deque.len() > SEQUENCE_LENGTH {
                    deque.pop_front();
                }
            }
        }

        // Check if history full; skip if not (avoids zero-padding obs)
        if self.input_features.iter().any(|kv| kv.value().len() < SEQUENCE_LENGTH) {
            info!(target: "strat::agent", "Skipping inference: insufficient history");
            return;
        }

        // Write to csv
        let mut row = Vec::with_capacity(NUM_FEATURES_OBS + 1);
        row.push(update.event_time.unix_timestamp().to_string());
        for feat_idx in 0..NUM_FEATURES_OBS {
            let feature_id = &self.input_features_ids[feat_idx];
            if let Some(deque) = self.input_features.get(feature_id) {
                row.push(deque.back().unwrap_or(&0.0).clone().to_string());
            }
        }
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("dump.csv")
            .unwrap();
        let mut buf_file = BufWriter::new(file);
        let is_new = buf_file.seek(SeekFrom::End(0)).unwrap() == 0; // Check if empty (new file)
        let mut wtr = Writer::from_writer(buf_file);

        if is_new {
            let mut header = vec!["event_time".to_string()];
            header.extend(self.input_features_ids.iter().map(|id| id.to_string()));
            wtr.serialize(&header).unwrap();
        }
        wtr.serialize(&row).unwrap();
        wtr.flush().unwrap();

        // Create Feature Input
        let mut input_data_flat_0: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS);
        for _batch in 0..BATCH_SIZE {
            for seq in 0..SEQUENCE_LENGTH {
                for feat_idx in 0..NUM_FEATURES_OBS {
                    let feature_id = &self.input_features_ids[feat_idx];
                    if let Some(deque) = self.input_features.get(feature_id) {
                        let len = deque.len();
                        let val = if seq >= SEQUENCE_LENGTH - len {
                            deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        input_data_flat_0.push(val);
                    } else {
                        input_data_flat_0.push(0.0);
                    }
                }
            }
        }

        // Create State Input
        let mut input_data_flat_1: Vec<f32> = Vec::with_capacity(BATCH_SIZE * SEQUENCE_LENGTH * NUM_STATE_OBS);
        let max_weight = POSSIBLE_WEIGHTS.iter().fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
        let scale_factor = 1.0 / (max_weight * 1.3489795003921636);
        for _batch in 0..BATCH_SIZE {
            for seq in 0..SEQUENCE_LENGTH {
                for state_idx in 0..NUM_STATE_OBS {
                    let state_id = &self.input_state_ids[state_idx];
                    if let Some(deque) = self.input_state.get(state_id) {
                        let len = deque.len();
                        let val = if seq >= SEQUENCE_LENGTH - len {
                            let weight = deque.get(seq - (SEQUENCE_LENGTH - len)).copied().unwrap_or(0.0);
                            weight * scale_factor
                        } else {
                            0.0
                        };
                        input_data_flat_1.push(val);
                    } else {
                        input_data_flat_1.push(0.0);
                    }
                }
            }
        }

        // Create Mask
        // Create Mask
        // let mask = if let Some(deque) = self.input_state.get(&self.input_state_ids[0]) {
        //     let current_weight = deque.back().copied().unwrap_or(0.0);
        //     self.create_mask(current_weight, &POSSIBLE_WEIGHTS)
        // } else {
        //     vec![true; POSSIBLE_WEIGHTS.len()] // Default: all actions allowed
        // };
        // let input_data_flat_2: Vec<bool> = (0..BATCH_SIZE).flat_map(|_| mask.clone()).collect();

        // Construct the inference request
        let inputs = vec![
            InferInputTensor {
                name: "INPUT__0".to_string(),
                shape: SHAPE_0.to_vec(),
                datatype: "FP32".to_string(),
                contents: Some(InferTensorContents {
                    fp32_contents: input_data_flat_0,
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            },
            InferInputTensor {
                name: "INPUT__1".to_string(),
                shape: SHAPE_1.to_vec(),
                datatype: "FP32".to_string(),
                contents: Some(InferTensorContents {
                    fp32_contents: input_data_flat_1,
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            },
            // InferInputTensor {
            //     name: "INPUT__2".to_string(),
            //     shape: SHAPE_2.to_vec(),
            //     datatype: "BOOL".to_string(),
            //     contents: Some(InferTensorContents {
            //         bool_contents: input_data_flat_2,
            //         ..Default::default()
            //     }),
            //     parameters: std::collections::HashMap::new(),
            // },
        ];

        let outputs = vec![
            InferRequestedOutputTensor {
                name: "OUTPUT__0".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "OUTPUT__1".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "OUTPUT__2".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            InferRequestedOutputTensor {
                name: "OUTPUT__3".to_string(),
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
                    warn!(target: "strat::agent", "inference failed: {}", e);
                    return;
                }
            }
        };

        let outputs = response.outputs;
        if outputs.is_empty() {
            warn!(target: "strat::agent", "No outputs in response");
            return;
        }
        // info!(target: "strat::agent", "{:?}", outputs);

        let mut outputs_map: HashMap<String, InferOutputTensor> = HashMap::new();
        for output in &outputs {
            outputs_map.insert(output.name.clone(), output.clone());
        }

        if let Some(weight_out) = outputs_map.get("OUTPUT__2") {
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
                let index = outputs.iter().position(|o| o.name == "OUTPUT__2").unwrap();
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

            info!(target: "strat::agent", "New weigth is {}", new_weight);

            // Now use new_weight: Push to deque
            if let Some(mut deque) = self.input_state.get_mut(&FeatureId::from("weight".to_owned())) {
                let prev_weight = deque.back().copied().unwrap_or(0.0);
                deque.push_back(new_weight);
                if deque.len() > SEQUENCE_LENGTH {
                    deque.pop_front();
                }
                let allocation_change = new_weight - prev_weight;
                if allocation_change != 0.0 {
                    let instrument = update.instruments[0].clone();
                    let quantity = Decimal::from_f32_retain(allocation_change.abs()).unwrap_or(Decimal::ZERO);
                    if quantity.is_zero() {
                        return;
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
                warn!(target: "strat::agent", "Missing weight deque");
                return;
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
