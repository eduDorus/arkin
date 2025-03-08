mod quantile_transform;
mod robust;

pub use quantile_transform::*;
pub use robust::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantileData {
    levels: Vec<f64>,
    data: Vec<QuantileEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantileEntryData {
    instrument_id: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}
