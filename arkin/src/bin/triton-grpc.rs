use std::time::Instant;

use arkin_core::{
    prelude::init_tracing,
    triton::{
        grpc_inference_service_client::GrpcInferenceServiceClient,
        model_infer_request::{InferInputTensor, InferRequestedOutputTensor},
        InferTensorContents, ModelInferRequest,
    },
};
use rand::{rng, Rng};
use tonic::{codec::CompressionEncoding, transport::Channel};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    init_tracing();

    info!("Starting GRPC Request");

    // Define constants
    const BATCH_SIZE: usize = 1;
    const SEQUENCE_LENGTH: usize = 192;
    const NUM_FEATURES_OBS: usize = 40;
    const NUM_FEATURES_STATE: usize = 3;
    const NUM_MASK: usize = 3;

    // Step 1: Generate and prepare the input data
    let mut rng = rng();
    let input_data_flat_0: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_OBS)
        .map(|_| rng.random::<f32>())
        .collect();
    let input_data_flat_1: Vec<f32> = (0..BATCH_SIZE * SEQUENCE_LENGTH * NUM_FEATURES_STATE)
        .map(|_| rng.random::<f32>())
        .collect();
    let input_data_flat_2: Vec<bool> = vec![true; BATCH_SIZE * NUM_MASK];

    // Define shapes
    let shape_0 = vec![BATCH_SIZE, SEQUENCE_LENGTH, NUM_FEATURES_OBS];
    let shape_1 = vec![BATCH_SIZE, SEQUENCE_LENGTH, NUM_FEATURES_STATE];
    let shape_2 = vec![BATCH_SIZE, NUM_MASK];

    let inputs = vec![
        InferInputTensor {
            name: "OBSERVATION".to_string(),
            shape: shape_0.iter().map(|&x| x as i64).collect(),
            datatype: "FP32".to_string(),
            contents: Some(InferTensorContents {
                fp32_contents: input_data_flat_0,
                ..Default::default()
            }),
            parameters: std::collections::HashMap::new(),
        },
        InferInputTensor {
            name: "STATE".to_string(),
            shape: shape_1.iter().map(|&x| x as i64).collect(),
            datatype: "FP32".to_string(),
            contents: Some(InferTensorContents {
                fp32_contents: input_data_flat_1,
                ..Default::default()
            }),
            parameters: std::collections::HashMap::new(),
        },
        InferInputTensor {
            name: "MASK".to_string(),
            shape: shape_2.iter().map(|&x| x as i64).collect(),
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

    let infer_request = ModelInferRequest {
        model_name: "agent".to_string(),
        model_version: "".to_string(),
        id: "".to_string(),
        parameters: std::collections::HashMap::new(),
        inputs,
        outputs,
        raw_input_contents: vec![],
    };

    let url = "http://192.168.100.100:8001";
    // let channel = match Channel::from_static(url).connect().await {
    //     Ok(c) => c,
    //     Err(e) => {
    //         error!("Connection error: {}", e);
    //         return;
    //     }
    // };
    let channel = Channel::from_static(url).connect_lazy();
    let mut client = GrpcInferenceServiceClient::new(channel).send_compressed(CompressionEncoding::Gzip); // Marginally slower with compression (but we will save some bandwith)
                                                                                                          // let mut client = GrpcInferenceServiceClient::new(channel); // Marginally faster without compression

    let start = Instant::now();

    for _ in 0..1440 {
        let start_inference = Instant::now();
        let response = match client.model_infer(infer_request.clone()).await {
            Ok(resp) => resp.into_inner(),
            Err(e) => {
                warn!("Inference failed: {}", e);
                info!("Total time: {:?}", start.elapsed());
                return;
            }
        };

        // Parse response (adapt your JSON handling)
        let outputs = response.outputs;
        if outputs.is_empty() {
            warn!("No outputs in response");
            return;
        }

        // for output in outputs {
        //     info!("{:?}", output);
        // }
        info!("Inference time: {:?}", start_inference.elapsed());
    }

    info!("Inference successful!");
    info!("Total time: {:?}", start.elapsed());
}
