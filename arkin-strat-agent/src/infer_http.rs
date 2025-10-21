#![allow(dead_code)]
use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::prelude::*;
use serde_json::json;
use tracing::error;

use crate::traits::InferenceService;

pub struct HTTPInferencer {
    base_url: String,
    client: Client,
}

impl HTTPInferencer {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Some(Duration::from_secs(300))) // Keep connections forever
            .build()
            .expect("Failed to create HTTP client");
        Self { base_url, client }
    }

    fn format_url(&self, model_name: &str) -> String {
        format!("{}/v2/models/{}/infer", self.base_url, model_name)
    }
}

#[async_trait]
impl InferenceService for HTTPInferencer {
    async fn request(
        &self,
        model_name: &str,
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
            .post(self.format_url(model_name))
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
