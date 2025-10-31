use anyhow::Result;
use ndarray::Array;
use ort::session::{builder::GraphOptimizationLevel, Session};

const SEQ_LEN: usize = 16;
const INPUTS_LEN: usize = 89;

fn main() -> Result<()> {
    run_ort()?;
    // run_candle()?;

    Ok(())
}

fn run_ort() -> Result<()> {
    // Build the ORT session
    let model = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(4)?
        .commit_from_file("../arkin-research/models/CNN-LSTM_trained.onnx")?;

    // Create a 3D input tensor with shape [1, SEQ_LEN, INPUTS_LEN] filled with 1.0
    let input_array = Array::from_elem((1, SEQ_LEN, INPUTS_LEN), 1.0f32);

    // Run the model with the correct input name "input"
    let timer = std::time::Instant::now();
    let mut counter = 0;
    for _ in 0..10000 {
        let input = ort::value::Tensor::from_array(input_array.clone().into_dyn())?;
        let outputs = model.run(ort::inputs!["input" => input].unwrap())?;
        let predictions = outputs["output"].try_extract_tensor::<f32>()?;
        info!("ORT Predictions: {:?}", predictions.as_slice());
        counter += 1;
    }
    info!("ORT time: {:?}", timer.elapsed() / counter);

    Ok(())
}
