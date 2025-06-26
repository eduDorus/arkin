use rand::Rng;
use serde_json::json;
use std::time::Instant;

const BATCH_SIZE: usize = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define dimensions (assuming example values since not provided)
    const FORECAST_SEQ_LEN: usize = 512;
    const FORECAST_NUM_FEATURES: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5

    // Step 1: Generate and prepare the input data
    let mut rng = rand::rng();
    let input_data_flat: Vec<f32> = (0..BATCH_SIZE * FORECAST_SEQ_LEN * FORECAST_NUM_FEATURES)
        .map(|_| rng.random::<f32>())
        .collect();

    // Define the shape
    let shape = vec![BATCH_SIZE, FORECAST_SEQ_LEN, FORECAST_NUM_FEATURES];

    // Step 2: Construct the inference request
    let infer_request = json!({
        "inputs": [
            {
                "name": "INPUT0",
                "shape": shape,
                "datatype": "FP32",
                "data": input_data_flat
            }
        ],
        "outputs": [
            {"name": "OUTPUT0"},
            {"name": "OUTPUT1"},
            {"name": "OUTPUT2"}
        ]
    });

    // Step 3: Send the request and handle the response
    let client = reqwest::Client::new();
    let url = "http://localhost:8000/v2/models/forecast/infer";

    let start = Instant::now();

    for _ in 0..10 {
        let response = client.post(url).json(&infer_request).send().await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let outputs = response_json["outputs"].as_array().unwrap();

            for (i, output) in outputs.iter().enumerate() {
                // let data = output["data"].as_array().unwrap();
                let shape = output["shape"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect::<Vec<usize>>();

                println!("Inference successful!");
                println!("OUTPUT{} shape: {:?}", i, shape);
            }
        } else {
            println!("Failed to get a response: {} - {}", response.status(), response.text().await?);
        }
    }

    let duration = start.elapsed();
    println!("Total time: {:?}", duration);

    // Define constants
    const SEQUENCE_LENGTH: usize = 36;
    const NUM_FEATURES_OBS: usize = 40; // Assuming len(FEATURE_COLUMNS) = 5
    const NUM_FEATURES_STATE: usize = 2;
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
    let client = reqwest::Client::new();
    let url = "http://localhost:8000/v2/models/agent/infer";

    let start = Instant::now();

    for _ in 0..10 {
        let response = client.post(url).json(&infer_request).send().await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let outputs = response_json["outputs"].as_array().ok_or("Outputs not an array")?;

            // Match Python output order: ACTION (0), ACTION_SPACE (1), PROBABILITY (3), WEIGHT (2)
            let output_order = vec![0, 1, 3, 2];
            let output_names = vec!["ACTION", "ACTION_SPACE", "WEIGHT", "PROBABILITY"];

            println!("Inference successful!");
            for (idx, &output_idx) in output_order.iter().enumerate() {
                let output = &outputs[output_idx];
                let shape = output["shape"]
                    .as_array()
                    .ok_or("Shape not an array")?
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect::<Vec<usize>>();
                let data = output["data"].as_array().ok_or("Data not an array")?;

                println!("{}: shape: {:?}, data length: {}", output_names[idx], shape, data.len());
                if !data.is_empty() {
                    println!("  First element: {}", data[0]);
                }
            }
        } else {
            println!("Failed to get a response: {} - {}", response.status(), response.text().await?);
        }
    }

    let duration = start.elapsed();
    println!("Total time: {:?}", duration);

    Ok(())
}
