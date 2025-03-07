use std::{collections::HashMap, fmt};

use anyhow::Result;
use clickhouse::{Client, Row};
use ort::info;
use serde::Deserialize;
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
    let levels: Vec<f64> = (1..n_quantiles).map(|i| i as f64 / n_quantiles as f64).collect();
    println!("Levels: {:?}", levels);
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

    let quantiles_data = client.query(&query).fetch_all::<ScalerData>().await?;
    println!("Rows: {}", quantiles_data.len());
    println!("First row: {}", quantiles_data[0]);

    // Initialize and fit the transformer
    let mut transformer = QuantileTransformer::new(1000, "uniform");
    transformer.fit(&quantiles_data);

    // Transform a new value
    let x = 0.00155;
    let feature_id = "volatility_60";
    let transformed_x = transformer.transform(x, feature_id);
    println!("Transformed value for x = {}: {}", x, transformed_x);
    let inverse_x = transformer.inverse_transform(transformed_x, feature_id);
    println!("Inverse transformed value for x = {}: {}", transformed_x, inverse_x);
    assert_eq!(x, inverse_x);

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

struct QuantileTransformer {
    feature_quantiles: HashMap<String, Vec<f64>>, // Quantiles per feature_id
    references: Vec<f64>,                         // Shared probability levels (e.g., 0 to 1)
    output_distribution: String,                  // "uniform" or "normal"
}

impl QuantileTransformer {
    /// Create a new transformer with a specified number of quantiles and output distribution
    fn new(n_quantiles: usize, output_distribution: &str) -> Self {
        let references = (0..n_quantiles).map(|i| i as f64 / n_quantiles as f64).collect();
        QuantileTransformer {
            feature_quantiles: HashMap::new(),
            references,
            output_distribution: output_distribution.to_string(),
        }
    }

    /// Fit the transformer with quantile data
    fn fit(&mut self, quantiles_data: &[ScalerData]) {
        for q in quantiles_data {
            self.feature_quantiles.insert(q.feature_id.clone(), q.quantiles.clone());
        }
    }

    /// Transform a value x for a given feature_id
    fn transform(&self, x: f64, feature_id: &str) -> f64 {
        let quantiles = self
            .feature_quantiles
            .get(feature_id)
            .expect("Feature ID not found in quantiles");
        let references = &self.references;

        // Forward interpolation
        let forward = interp(x, quantiles, references);

        // Reverse interpolation
        let quantiles_rev: Vec<f64> = quantiles.iter().rev().map(|&v| -v).collect();
        let references_rev: Vec<f64> = references.iter().rev().map(|&v| -v).collect();
        let reverse = interp(-x, &quantiles_rev, &references_rev);

        // Average the two interpolations
        let p = 0.5 * (forward - reverse);

        // Apply the output distribution
        if self.output_distribution == "uniform" {
            p
        } else if self.output_distribution == "normal" {
            let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
            let clip_min = normal.inverse_cdf(1e-7); // Avoid -infinity
            let clip_max = normal.inverse_cdf(1.0 - 1e-7); // Avoid +infinity
            normal.inverse_cdf(p).max(clip_min).min(clip_max)
        } else {
            panic!("Unsupported output distribution: {}", self.output_distribution);
        }
    }

    pub fn inverse_transform(&self, y: f64, feature_id: &str) -> f64 {
        // Step 1: Get quantiles and references for the feature
        let quantiles = self.feature_quantiles.get(feature_id).expect("Feature ID not found");
        let references = &self.references;

        // Step 2: Compute p based on output distribution
        let p = if self.output_distribution == "normal" {
            let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
            normal.cdf(y)
        } else {
            y
        };

        // Step 3: Interpolate p to get x
        interp(p, references, quantiles)
    }
}

#[derive(Row, Deserialize)]
struct ScalerData {
    #[serde(with = "clickhouse::serde::uuid")]
    instrument: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}

impl fmt::Display for ScalerData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Instrument: {}, Feature: {}, Quantiles: {} Median: {} IQR: {}",
            self.instrument,
            self.feature_id,
            self.quantiles.len(),
            self.median,
            self.iqr,
        )
    }
}
