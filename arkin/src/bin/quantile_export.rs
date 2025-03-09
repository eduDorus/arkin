use anyhow::Result;
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
    let levels: Vec<f64> = (1..n_quantiles).map(|i| i as f64 / n_quantiles as f64).collect();

    let levels_str = levels.iter().map(|&l| format!("{:.4}", l)).collect::<Vec<_>>().join(", ");
    let query = format!(
        r#"
        SELECT
          instrument_id,
          feature_id,
          quantilesExact({})(value) AS quantiles,
          quantileExact(0.5)(value) AS median,
          (quantileExact(0.75)(value) - quantileExact(0.25)(value)) as iqr
        FROM
          insights FINAL
        WHERE
          event_time BETWEEN '2021-01-02 00:00:00' AND '2025-01-01 00:00:00'
          AND pipeline_id = 'd0e2617c-3682-4d6d-8f6e-d499a772dec6'
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
    println!("{}", query);

    let quantiles_data = client.query(&query).fetch_all::<ScalerDataDTO>().await?;
    let scaler_data = QuantileData {
        levels,
        data: quantiles_data.into_iter().map(|q| q.into()).collect(),
    };

    // Save to json file:
    let file = std::fs::File::create("./scalers/quantiles.json")?;
    serde_json::to_writer_pretty(file, &scaler_data)?;

    // Load from json file:
    // let file = std::fs::File::open("./scalers/quantile_scaler.json")?;
    // let scaler_data: QuantileData = serde_json::from_reader(file)?;

    Ok(())
}

fn _get_exact_iqr() -> f64 {
    let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
    let q1 = normal.inverse_cdf(0.25);
    let q3 = normal.inverse_cdf(0.75);
    q3 - q1
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct ScalerDataDTO {
    #[serde(with = "clickhouse::serde::uuid")]
    instrument_id: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QuantileData {
    levels: Vec<f64>,
    data: Vec<ScalerDataDTO>,
}
