use std::sync::Arc;

use clickhouse::{query::RowCursor, sql::Identifier, Row};
use futures::channel;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};
use tracing::info;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::context::PersistenceContext;

const TABLE_NAME: &str = "metrics";

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct MetricsClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub metric_type: String,
    #[serde(with = "custom_serde::decimal64")]
    pub value: Decimal,
}

impl From<Arc<Metric>> for MetricsClickhouseDTO {
    fn from(metric: Arc<Metric>) -> Self {
        Self {
            event_time: metric.event_time.into(),
            instrument_id: metric.instrument.id,
            metric_type: metric.metric_type.to_string(),
            value: metric.value,
        }
    }
}

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    ctx.ch_client
        .query(
            "
                CREATE TABLE IF NOT EXISTS ?
                (
                    event_time      DateTime64(3, 'UTC') CODEC(Delta, ZSTD(3)),
                    instrument_id   LowCardinality(UUID) CODEC(ZSTD(3)),
                    metric_type     LowCardinality(String) CODEC(ZSTD(3)),
                    value        	  Decimal(18, 8) CODEC(ZSTD(3))
                )
                ENGINE = ReplacingMergeTree
                PARTITION BY toYYYYMM(event_time)
                ORDER BY (instrument_id, metric_type, event_time)
                SETTINGS index_granularity = 8192;
                ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, metric: MetricsClickhouseDTO) -> Result<(), PersistenceError> {
    info!(target: "persistence", "inserting metric: {:?}", metric);
    let mut insert = ctx.ch_client.insert::<MetricsClickhouseDTO>(TABLE_NAME).await?;
    insert.write(&metric).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, metrics: &[MetricsClickhouseDTO]) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<MetricsClickhouseDTO>(TABLE_NAME).await?;
    for metric in metrics {
        insert.write(metric).await?;
    }
    insert.end().await?;
    Ok(())
}

pub async fn stream_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    metric_type: MetricType,
    start: UtcDateTime,
    end: UtcDateTime,
) -> Result<RowCursor<MetricsClickhouseDTO>, PersistenceError> {
    let cursor = ctx
        .ch_client
        .query(
            r#"
                SELECT 
                ?fields 
            FROM ? 
            WHERE 
                event_time BETWEEN ? AND ? 
                AND instrument_id IN (?) 
                AND metric_type = ?
            ORDER BY 
                event_time ASC
        "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(start.unix_timestamp())
        .bind(end.unix_timestamp())
        .bind(instrument_ids)
        .bind(metric_type.to_string())
        .fetch::<MetricsClickhouseDTO>()?;
    Ok(cursor)
}

pub async fn fetch_batch(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    metric_type: MetricType,
    start: UtcDateTime,
    end: UtcDateTime,
) -> Result<Vec<MetricsClickhouseDTO>, PersistenceError> {
    let rows = ctx
        .ch_client
        .query(
            r#"
            SELECT 
                ?fields 
            FROM ? 
            WHERE 
                event_time BETWEEN ? AND ? 
                AND instrument_id IN (?) 
                AND metric_type = ?
            ORDER BY 
                event_time ASC
        "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(start.unix_timestamp())
        .bind(end.unix_timestamp())
        .bind(instrument_ids)
        .bind(metric_type.to_string())
        .fetch_all::<MetricsClickhouseDTO>()
        .await?;

    info!(target: "persistence", "fetched {} metrics", rows.len());

    Ok(rows)
}
