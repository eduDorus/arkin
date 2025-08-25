#![allow(dead_code)]
use std::collections::HashMap;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{debug, error, warn};

use arkin_core::prelude::*;

use crate::traits::InferenceService;

// let url = "http://192.168.100.100:8001";

pub struct GrpcInferencer {
    url: String,
    client: Mutex<GrpcInferenceServiceClient<Channel>>,
}

impl GrpcInferencer {
    pub fn new(url: &str) -> Self {
        let channel = Channel::from_shared(url.to_string())
            .expect("invalid gRPC server URL")
            .connect_lazy();
        let client = Mutex::new(GrpcInferenceServiceClient::new(channel));
        Self {
            url: url.to_string(),
            client,
        }
    }
}

#[async_trait]
impl InferenceService for GrpcInferencer {
    async fn request(
        &self,
        model_name: &str,
        input_names: &[&str],
        input_values: &[&[f32]],
        shapes: &[&[i64]],
        output_names: &[&str],
    ) -> Option<HashMap<String, Vec<Decimal>>> {
        // Build the request
        let inputs = input_names
            .iter()
            .zip(input_values.iter())
            .zip(shapes)
            .map(|((name, value), shape)| InferInputTensor {
                name: name.to_string(),
                shape: shape.to_vec(),
                datatype: "FP32".to_string(),
                contents: Some(InferTensorContents {
                    fp32_contents: value.to_vec(),
                    ..Default::default()
                }),
                parameters: std::collections::HashMap::new(),
            })
            .collect();

        let outputs = output_names
            .iter()
            .map(|name| InferRequestedOutputTensor {
                name: name.to_string(),
                parameters: std::collections::HashMap::new(),
            })
            .collect();

        // Build the inference request
        let infer_request = ModelInferRequest {
            model_name: model_name.to_string(),
            model_version: "".to_string(),
            id: "".to_string(),
            parameters: std::collections::HashMap::new(),
            inputs,
            outputs,
            raw_input_contents: vec![],
        };

        // Send the request
        match self.client.lock().await.model_infer(infer_request).await {
            Ok(response) => {
                let response = response.into_inner();

                let outputs = response.outputs;
                if outputs.is_empty() {
                    error!(target: "strat::agent", "No outputs in response");
                    return None;
                }

                debug!(target: "strat::agent", "Received outputs: {:?}", outputs);

                // Parse outputs to a hashmap
                let mut result: HashMap<String, Vec<Decimal>> = HashMap::new();
                let use_raw = !response.raw_output_contents.is_empty();
                for (idx, output) in outputs.into_iter().enumerate() {
                    let name = output.name.clone();
                    let datatype = &output.datatype;
                    let num_elements: usize = output.shape.iter().product::<i64>() as usize;
                    let mut vec_dec: Vec<Decimal> = Vec::with_capacity(num_elements);
                    if use_raw {
                        let raw_bytes = &response.raw_output_contents[idx];
                        let elem_size = match datatype.as_str() {
                            "FP32" => 4,
                            "INT64" => 8,
                            _ => {
                                warn!(target: "strat::agent", "Unsupported datatype for raw parsing '{}': {}", name, datatype);
                                continue;
                            }
                        };
                        if raw_bytes.len() != num_elements * elem_size {
                            warn!(target: "strat::agent", "Invalid raw bytes length for '{}': {} (expected {})", name, raw_bytes.len(), num_elements * elem_size);
                            continue;
                        }
                        for i in (0..raw_bytes.len()).step_by(elem_size) {
                            let bytes_slice = &raw_bytes[i..i + elem_size];
                            let dec = match datatype.as_str() {
                                "FP32" => {
                                    let bytes: [u8; 4] = bytes_slice.try_into().expect("FP32 byte slice mismatch");
                                    let val = f32::from_le_bytes(bytes);
                                    Decimal::from_f32(val).unwrap_or(Decimal::ZERO)
                                }
                                "INT64" => {
                                    let bytes: [u8; 8] = bytes_slice.try_into().expect("INT64 byte slice mismatch");
                                    let val = i64::from_le_bytes(bytes);
                                    Decimal::from_i64(val).unwrap_or(Decimal::ZERO)
                                }
                                _ => unreachable!(),
                            };
                            vec_dec.push(dec);
                        }
                    } else if let Some(contents) = output.contents {
                        match datatype.as_str() {
                            "FP32" => {
                                let vals = contents.fp32_contents;
                                if vals.len() != num_elements {
                                    warn!(target: "strat::agent", "Invalid fp32 contents length for '{}': {} (expected {})", name, vals.len(), num_elements);
                                    continue;
                                }
                                vec_dec = vals.iter().map(|&f| Decimal::from_f32(f).unwrap_or(Decimal::ZERO)).collect();
                            }
                            "INT64" => {
                                let vals = contents.int64_contents;
                                if vals.len() != num_elements {
                                    warn!(target: "strat::agent", "Invalid int64 contents length for '{}': {} (expected {})", name, vals.len(), num_elements);
                                    continue;
                                }
                                vec_dec = vals.iter().map(|&i| Decimal::from_i64(i).unwrap_or(Decimal::ZERO)).collect();
                            }
                            _ => {
                                warn!(target: "strat::agent", "Unsupported datatype for contents parsing '{}': {}", name, datatype);
                                continue;
                            }
                        }
                    } else {
                        warn!(target: "strat::agent", "No contents or raw for '{}'", name);
                        continue;
                    }
                    result.insert(name, vec_dec);
                }
                if result.is_empty() {
                    error!(target: "strat::agent", "No valid outputs parsed");
                    return None;
                }
                Some(result)
            }
            Err(e) => {
                error!("Inference failed: {}", e);
                None
            }
        }
    }
}
