use arkin_core::prelude::init_tracing;
use rand::Rng;
use serde_json::json;
use std::time::Instant;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Define constants
    const BATCH_SIZE: usize = 1;
    const SEQUENCE_LENGTH: usize = 192;
    const NUM_FEATURES_OBS: usize = 40;
    const NUM_FEATURES_STATE: usize = 3;
    const NUM_MASK: usize = 3;

    // Step 1: Generate and prepare the input data
    let mut rng = rand::rng();
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
    let client = reqwest::Client::builder()
        .pool_idle_timeout(None) // Keep connections forever
        .build()?;
    let url = "http://192.168.100.100:8000/v2/models/agent/infer";

    let start = Instant::now();

    for _ in 0..10 {
        let start_inference = Instant::now();

        let response = client.post(url).json(&infer_request).send().await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let outputs = response_json["outputs"].as_array().ok_or("Outputs not an array")?;

            // Match Python output order: ACTION (0), ACTION_SPACE (1), PROBABILITY (3), WEIGHT (2)
            let output_order = [0, 1, 3, 2];
            // let output_names = ["ACTION", "ACTION_SPACE", "WEIGHT", "PROBABILITY"];

            for (_idx, &output_idx) in output_order.iter().enumerate() {
                let output = &outputs[output_idx];
                info!("{:?}", output);
            }
        } else {
            info!("Failed to get a response: {} - {}", response.status(), response.text().await?);
        }
        info!("Inference time: {:?}", start_inference.elapsed());
    }

    info!("Inference successful!");
    info!("Total time: {:?}", start.elapsed());

    Ok(())
}
