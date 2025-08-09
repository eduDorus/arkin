use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use serde_json::json;
use tracing::{instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

#[derive(TypedBuilder)]
pub struct Forecast {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    client: reqwest::Client,
}

impl Forecast {
    async fn insight_tick(&self, _tick: &InsightsUpdate) {
        // Define constants
        const BATCH_SIZE: usize = 1;
        const FORECAST_SEQ_LEN: usize = 512;
        const FORECAST_NUM_FEATURES: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5

        // Step 1: Generate and prepare the input data
        let input_data: Vec<f32> = (0..BATCH_SIZE * FORECAST_SEQ_LEN * FORECAST_NUM_FEATURES)
            .map(|_| 0.0)
            .collect();

        // Define shapes
        let shape = vec![BATCH_SIZE, FORECAST_SEQ_LEN, FORECAST_NUM_FEATURES];

        // Step 2: Construct the inference request
        let infer_request = json!({
            "inputs": [
                {
                    "name": "INPUT0",
                    "shape": shape,
                    "datatype": "FP32",
                    "data": input_data
                }
            ],
            "outputs": [
                {"name": "OUTPUT0"},
                {"name": "OUTPUT1"},
                {"name": "OUTPUT2"}
            ]
        });

        // Step 3: Send the request and handle the response
        let url = "http://localhost:8000/v2/models/forecast/infer";
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
        let output_order = [0, 1, 2];
        let output_names = ["OUTPUT0", "OUTPUT1", "OUTPUT2"];

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
impl Runnable for Forecast {
    async fn handle_event(&self, _ctx: Arc<CoreServices>, event: Event) {
        match &event {
            Event::InsightsUpdate(vo) => self.insight_tick(vo).await,
            e => warn!(target: "forecast", "received unused event {}", e),
        }
    }
}
