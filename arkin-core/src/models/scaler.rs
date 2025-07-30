use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantileData {
    pub instrument_id: Uuid,
    pub feature_id: String,
    pub quantiles: Vec<f64>,
    pub median: f64,
    pub iqr: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Quantiles {
    pub pipeline_id: Uuid,
    pub levels: Vec<f64>,
    pub data: Vec<QuantileData>,
}
