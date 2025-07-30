use std::{collections::HashMap, sync::Arc};

use clickhouse::{query::RowCursor, sql::Identifier, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};
use tracing::info;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, PersistenceError};

#[derive(Debug, Serialize, Deserialize, Row, Clone)]
struct ScalerDataDTO {
    #[serde(with = "clickhouse::serde::uuid")]
    instrument_id: Uuid,
    feature_id: String,
    quantiles: Vec<f64>,
    median: f64,
    iqr: f64,
}

pub async fn get_iqr(
    ctx: &PersistenceContext,
    pipeline_id: Uuid,
    instrument_id: Uuid,
    from: UtcDateTime,
    till: UtcDateTime,
    levels: &[f64],
) -> Result<Vec<QuantileData>, PersistenceError> {
    let levels_str = levels.iter().map(|&l| format!("{l:.4}")).collect::<Vec<_>>().join(", ");

    let from = from.unix_timestamp();
    let till = till.unix_timestamp();

    // Get distinct feature_ids:
    let feature_query = format!(
        r#"
        SELECT 
          DISTINCT feature_id
        FROM 
          insights FINAL
        WHERE
          event_time BETWEEN '{from}' AND '{till}'
          AND pipeline_id = '{pipeline_id}'
          AND instrument_id = '{instrument_id}'
          AND insight_type = 'continuous'
        ORDER BY 
          feature_id ASC
        "#
    );
    let feature_ids: Vec<String> = ctx.ch_client.query(&feature_query).fetch_all::<String>().await?;
    for feature in &feature_ids {
        info!(target: "persistance", " - {feature}");
    }

    // Create a query for each feature_id:
    let mut quantile_result = Vec::with_capacity(feature_ids.len());
    for feature_id in feature_ids {
        let query = format!(
            r#"
            SELECT
              instrument_id,
              feature_id,
              quantilesExact({levels_str})(value) AS quantiles,
              quantileExact(0.5)(value) AS median,
              (quantileExact(0.75)(value) - quantileExact(0.25)(value)) as iqr
            FROM
              insights FINAL
            WHERE
              event_time BETWEEN '{from}' AND '{till}'
              AND pipeline_id = '{pipeline_id}'
              AND instrument_id = '{instrument_id}'
              AND insight_type = 'continuous'
              AND feature_id = '{feature_id}'
            GROUP BY
              instrument_id,
              feature_id
            ORDER BY
              instrument_id,
              feature_id ASC
            "#
        );
        let quantile_data = ctx.ch_client.query(&query).fetch_one::<ScalerDataDTO>().await?;
        let data = QuantileData {
            instrument_id: instrument_id,
            feature_id: feature_id,
            quantiles: quantile_data.quantiles,
            median: quantile_data.median,
            iqr: quantile_data.iqr,
        };
        quantile_result.push(data);
    }

    Ok(quantile_result)
}
