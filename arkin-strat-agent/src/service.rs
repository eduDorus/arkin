use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use serde_json::json;
use tracing::{instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

#[derive(TypedBuilder)]
pub struct AgentStrategy {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    client: reqwest::Client,
}

impl AgentStrategy {
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn insight_tick(&self, _tick: &InsightsUpdate) {
        // Define constants
        const BATCH_SIZE: usize = 1;
        const SEQUENCE_LENGTH: usize = 36;
        const NUM_FEATURES_OBS: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5
        const NUM_FEATURES_STATE: usize = 2;
        const NUM_MASK: usize = 3;

        // Step 1: Generate and prepare the input data
        let input_data_flat_0: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS).map(|_| 0.0).collect();
        let input_data_flat_1: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_STATE).map(|_| 0.0).collect();
        let input_data_flat_2: Vec<bool> = vec![true; BATCH_SIZE * NUM_MASK];

        // Define shapes
        let shape_0 = vec![BATCH_SIZE, SEQUENCE_LENGTH, NUM_FEATURES_OBS];
        let shape_1 = vec![BATCH_SIZE, SEQUENCE_LENGTH, NUM_FEATURES_STATE];
        let shape_2 = vec![BATCH_SIZE, NUM_MASK];

        // Step 2: Construct the inference request
        let infer_request = json!({
            "inputs": [
                {
                    "name": "OBSERVATION",
                    "shape": shape_0,
                    "datatype": "FP32",
                    "data": input_data_flat_0
                },
                {
                    "name": "STATE",
                    "shape": shape_1,
                    "datatype": "FP32",
                    "data": input_data_flat_1
                },
                {
                    "name": "MASK",
                    "shape": shape_2,
                    "datatype": "BOOL",
                    "data": input_data_flat_2
                }
            ],
            "outputs": [
                {"name": "ACTION"},
                {"name": "ACTION_SPACE"},
                {"name": "WEIGHT"},
                {"name": "PROBABILITY"}
            ]
        });

        // Step 3: Send the request and handle the response
        let url = "http://localhost:8000/v2/models/agent/infer";
        let start = Instant::now();

        let response = match self.client.post(url).json(&infer_request).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to send inference request: {}", e);
                println!("Total time: {:?}", start.elapsed());
                return;
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body = match response.text().await {
                Ok(text) => text,
                Err(e) => format!("Failed to read error body: {}", e),
            };
            warn!("Inference request failed with status {}: {}", status, body);
            println!("Total time: {:?}", start.elapsed());
            return;
        }

        let response_json: serde_json::Value = match response.json().await {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to parse inference response as JSON: {}", e);
                println!("Total time: {:?}", start.elapsed());
                return;
            }
        };

        let Some(outputs) = response_json["outputs"].as_array() else {
            warn!("Inference response 'outputs' is not an array");
            println!("Total time: {:?}", start.elapsed());
            return;
        };

        // Match Python output order: ACTION (0), ACTION_SPACE (1), PROBABILITY (3), WEIGHT (2)
        let output_order = [0, 1, 3, 2];
        let output_names = ["ACTION", "ACTION_SPACE", "WEIGHT", "PROBABILITY"];

        println!("Inference successful!");
        for (idx, &output_idx) in output_order.iter().enumerate() {
            if output_idx >= outputs.len() {
                warn!("Output index {} out of bounds in response array", output_idx);
                continue;
            }
            let output = &outputs[output_idx];

            let shape: Vec<usize> = match output["shape"].as_array() {
                Some(arr) => arr.iter().filter_map(|v| v.as_u64().map(|u| u as usize)).collect(),
                None => {
                    warn!("Output '{}' shape is not an array", output_names[idx]);
                    continue;
                }
            };

            let Some(data) = output["data"].as_array() else {
                warn!("Output '{}' data is not an array", output_names[idx]);
                continue;
            };

            println!("{}: shape: {:?}, data length: {}", output_names[idx], shape, data.len());
            if !data.is_empty() {
                println!("  First element: {}", data[0]);
            }
        }

        println!("Total time: {:?}", start.elapsed());
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
            e => warn!(target: "strat-crossover", "received unused event {}", e),
        }
    }
}
