use rand::Rng;
use reqwest::Client;
use rust_decimal::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::{error, info};

use arkin_core::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Create HTTP client
    let url = "http://192.168.100.100:8000/v2/models/agent/versions/1/infer";
    let client = HTTPInferencer::new(url.to_string());

    // Define constants
    const BATCH_SIZE: usize = 1;
    const SEQUENCE_LENGTH: usize = 192;
    const NUM_FEATURES_OBS: usize = 40;
    const NUM_STATE_OBS: usize = 1;

    const SHAPE_0: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_FEATURES_OBS as i64];
    const SHAPE_1: [i64; 3] = [BATCH_SIZE as i64, SEQUENCE_LENGTH as i64, NUM_STATE_OBS as i64];

    const OUTPUT_WEIGHT_NAME: &str = "OUTPUT__2";

    // Step 1: Generate and prepare the input data
    let input_names = vec!["INPUT__0", "INPUT__1"];
    let mut rng = rand::rng();
    let input_data_0: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS)
        .map(|_| rng.random::<f32>())
        .collect();
    let input_data_1: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_STATE_OBS)
        .map(|_| rng.random::<f32>())
        .collect();
    let output_names = vec!["OUTPUT__0", "OUTPUT__1", "OUTPUT__2", "OUTPUT__3"];

    let start = Instant::now();

    for _ in 0..10 {
        let start_inference = Instant::now();

        let response = client
            .request(
                &input_names,
                &[&input_data_0, &input_data_1],
                &[&SHAPE_0, &SHAPE_1],
                &output_names,
            )
            .await;

        if let Some(outputs) = response {
            info!("{:?}", outputs);
            let new_weight = outputs.get(OUTPUT_WEIGHT_NAME).cloned().unwrap_or_default()[0];
            info!("New weight: {:?}", new_weight);
        } else {
            error!("Failed to get a response");
        }
        info!("Inference time: {:?}", start_inference.elapsed());
    }

    info!("Inference successful!");
    info!("Total time: {:?}", start.elapsed());

    Ok(())
}

pub struct HTTPInferencer {
    url: String,
    client: Client,
}

impl HTTPInferencer {
    pub fn new(url: String) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(300))) // Keep connections forever
            .build()
            .expect("Failed to create HTTP client");
        Self { url, client }
    }

    pub async fn request(
        &self,
        input_names: &[&str],
        input_values: &[&[f32]],
        shapes: &[&[i64]],
        output_names: &[&str],
    ) -> Option<HashMap<String, Vec<Decimal>>> {
        // We need to build the request json
        let inputs_json: Vec<_> = input_names
            .iter()
            .zip(input_values.iter())
            .zip(shapes)
            .map(|((input_name, input_value), shape)| {
                json!({
                    "name": input_name,
                    "shape": shape,
                    "datatype": "FP32",
                    "data": input_value
                })
            })
            .collect();

        let outputs_json: Vec<_> = output_names
            .iter()
            .map(|output| {
                json!({
                    "name": output
                })
            })
            .collect();

        let infer_request = json!({
            "inputs": inputs_json,
            "outputs": outputs_json
        });

        // Send the request
        let res = self
            .client
            .post(&self.url)
            .json(&infer_request)
            .send()
            .await
            .expect("Failed to send inference request");

        // Handle the response
        if res.status().is_success() {
            let response_json: serde_json::Value = res.json().await.expect("Failed to parse response");
            let res = response_json["outputs"].as_array().expect("Outputs not an array");

            // Process the successful response
            let mut outputs = HashMap::new();
            for output in res.iter() {
                // Get the name and the value
                // The value must be converted from Number to Decimal
                let name = output["name"].as_str().expect("Output name not a string");
                let value = output["data"]
                    .as_array()
                    .expect("Output data not an array")
                    .iter()
                    .map(|v| v.as_f64().and_then(Decimal::from_f64).unwrap_or(Decimal::ZERO))
                    .collect::<Vec<Decimal>>();
                outputs.insert(name.to_string(), value.clone());
            }
            Some(outputs)
        } else {
            error!("Inference request failed: {}", res.status());
            None
        }
    }
}
