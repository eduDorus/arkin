use std::{collections::HashMap, fmt};

use anyhow::Result;
use arkin_core::FeatureId;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_compression(clickhouse::Compression::Lz4)
        .with_database("arkin")
        .with_user("arkin_admin")
        .with_password("test1234")
        .with_option("wait_end_of_query", "1");

    let n_quantiles = 1000;
    let levels: Vec<f64> = (0..=n_quantiles).map(|i| i as f64 / n_quantiles as f64).collect();

    let levels_str = levels.iter().map(|&l| format!("{:.4}", l)).collect::<Vec<_>>().join(", ");
    let query = format!(
        r#"
        SELECT
          instrument_id,
          feature_id,
          quantilesExact({})(value) AS quantiles,
          quantileExact(0.5)(value) AS median,
          (quantileExact(0.75)(value) - quantileExact(0.025)(value)) as iqr
        FROM
          insights FINAL
        WHERE
          event_time BETWEEN '2024-06-01 00:00:00' AND '2025-01-01 00:00:00'
          AND pipeline_id = 'b270ab95-a382-4d15-ac7a-d714cdb14037'
          AND insight_type = 'continuous'
        GROUP BY
          instrument_id,
          feature_id
        ORDER BY
          instrument_id,
          feature_id ASC
        "#,
        levels_str
    );

    let quantiles_data = client.query(&query).fetch_all::<ScalerDataDTO>().await?;
    let scaler_data = QuantileData {
        levels,
        data: quantiles_data.into_iter().map(|q| q.into()).collect(),
    };

    // Save to json file:
    let file = std::fs::File::create("./scalers/quantile_scaler.json")?;
    serde_json::to_writer_pretty(file, &scaler_data)?;

    // Load from json file:
    let file = std::fs::File::open("./scalers/quantile_scaler.json")?;
    let scaler_data: QuantileData = serde_json::from_reader(file)?;

    // Initialize and fit the transformer
    let transformer = QuantileTransformer::new(scaler_data.clone(), DistributionType::Normal);
    let scaler = RobustScaler::new(scaler_data);

    // Transform a new value
    let instrument_id = Uuid::parse_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6")?;
    let x = 0.3885196962635255;
    let feature_id = "adx_11".to_string().into();

    let transformed_x = transformer.transform(instrument_id, &feature_id, x);
    println!("Transformed value for x = {}: {}", x, transformed_x);

    let transformed_scaled_x = scaler.transform(instrument_id, &feature_id, transformed_x);
    println!("Scaled value for x = {}: {}", transformed_x, transformed_scaled_x);

    let transformed_x = scaler.inverse_transform(instrument_id, &feature_id, transformed_scaled_x);
    println!("Inverse scaled value for x = {}: {}", transformed_scaled_x, transformed_x);

    let original_x = transformer.inverse_transform(instrument_id, &feature_id, transformed_x);
    println!("Inverse transformed value for x = {}: {}", transformed_x, original_x);

    assert!(x - original_x < 1e-8);

    Ok(())
}

// Linear interpolation function
fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    assert!(
        xp.len() == fp.len() && xp.len() >= 2,
        "xp and fp must have same length and at least 2 elements"
    );
    if x <= xp[0] {
        fp[0] // Return the lower bound if x is below the smallest quantile
    } else if x >= xp[xp.len() - 1] {
        fp[fp.len() - 1] // Return the upper bound if x is above the largest quantile
    } else {
        let i = xp.iter().position(|&v| v > x).unwrap() - 1;
        let x0 = xp[i];
        let x1 = xp[i + 1];
        let f0 = fp[i];
        let f1 = fp[i + 1];
        f0 + (x - x0) * (f1 - f0) / (x1 - x0) // Linear interpolation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistributionType {
    Uniform,
    Normal,
}

struct QuantileTransformer {
    feature_quantiles: HashMap<(Uuid, FeatureId), Vec<f64>>, // Quantiles per feature_id
    references: Vec<f64>,                                    // Shared probability levels (e.g., 0 to 1)
    output_distribution: DistributionType,                   // "uniform" or "normal"
}

impl QuantileTransformer {
    /// Create a new transformer with a specified number of quantiles and output distribution
    fn new(scaler_data: QuantileData, output_distribution: DistributionType) -> Self {
        let references = scaler_data.levels;
        let feature_quantiles = scaler_data
            .data
            .into_iter()
            .map(|q| ((q.instrument_id, q.feature_id.into()), q.quantiles))
            .collect();
        QuantileTransformer {
            feature_quantiles: feature_quantiles,
            references,
            output_distribution: output_distribution,
        }
    }

    /// Transform a value x for a given feature_id
    fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let quantiles = self.feature_quantiles.get(&key).expect("Feature ID not found in quantiles");
        // Forward interpolation
        let forward = interp(x, quantiles, &self.references);

        // Reverse interpolation
        let quantiles_rev: Vec<f64> = quantiles.iter().rev().map(|&v| -v).collect();
        let references_rev: Vec<f64> = self.references.iter().rev().map(|&v| -v).collect();
        let reverse = interp(-x, &quantiles_rev, &references_rev);

        // Average the two interpolations
        let p = 0.5 * (forward - reverse);

        // Apply the output distribution
        match self.output_distribution {
            DistributionType::Uniform => p,
            DistributionType::Normal => {
                let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
                let clip_min = normal.inverse_cdf(1e-7); // Avoid -infinity
                let clip_max = normal.inverse_cdf(1.0 - 1e-7); // Avoid +infinity
                normal.inverse_cdf(p).max(clip_min).min(clip_max)
            }
        }
    }

    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, y: f64) -> f64 {
        // Step 1: Get quantiles and references for the feature
        let key = (instrument_id, feature_id.clone());
        let quantiles = self.feature_quantiles.get(&key).expect("Feature ID not found");

        // Step 2: Compute p based on output distribution
        let p = match self.output_distribution {
            DistributionType::Uniform => y,
            DistributionType::Normal => {
                let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
                normal.cdf(y)
            }
        };

        // Step 3: Interpolate p to get x
        interp(p, &self.references, quantiles)
    }
}

pub struct RobustScaler {
    feature_data: HashMap<(Uuid, FeatureId), (f64, f64)>,
}

impl RobustScaler {
    pub fn new(scaler_data: QuantileData) -> Self {
        let feature_data = scaler_data
            .data
            .into_iter()
            .map(|q| {
                let key = (q.instrument_id, q.feature_id.into());
                let value = (q.median, q.iqr);
                (key, value)
            })
            .collect();
        RobustScaler { feature_data }
    }

    pub fn transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self.feature_data.get(&key).expect("Feature ID not found");
        (x - median) / iqr
    }

    pub fn inverse_transform(&self, instrument_id: Uuid, feature_id: &FeatureId, x: f64) -> f64 {
        let key = (instrument_id, feature_id.clone());
        let (median, iqr) = self.feature_data.get(&key).expect("Feature ID not found");
        x * iqr + median
    }
}

#[derive(Debug, Clone, Row, Deserialize)]
struct ScalerDataDTO {
    #[serde(with = "clickhouse::serde::uuid")]
    instrument: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}

impl From<ScalerDataDTO> for QuantileEntryData {
    fn from(dto: ScalerDataDTO) -> Self {
        QuantileEntryData {
            instrument_id: dto.instrument,
            feature_id: dto.feature_id,
            quantiles: dto.quantiles,
            median: dto.median,
            iqr: dto.iqr,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantileData {
    levels: Vec<f64>,
    data: Vec<QuantileEntryData>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantileEntryData {
    instrument_id: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}

impl fmt::Display for QuantileEntryData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Instrument: {}, Feature: {}, Quantiles: {} Median: {} IQR: {}",
            self.instrument_id,
            self.feature_id,
            self.quantiles.len(),
            self.median,
            self.iqr,
        )
    }
}
