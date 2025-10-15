use std::sync::Arc;

use arkin_core::Insight;
use clickhouse::{sql::Identifier, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

const TABLE_NAME: &str = "insights";

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct InsightClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub pipeline_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub feature_id: String,
    pub value: f64,
    pub insight_type: String,
}

impl From<Arc<Insight>> for InsightClickhouseDTO {
    fn from(insight: Arc<Insight>) -> Self {
        Self {
            event_time: insight.event_time.into(),
            pipeline_id: insight.pipeline.as_ref().map(|p| p.id).unwrap(),
            instrument_id: insight.instrument.as_ref().map(|i| i.id).unwrap(),
            feature_id: insight.feature_id.to_string(),
            value: insight.value,
            insight_type: insight.insight_type.to_string(),
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
                    pipeline_id     UUID CODEC(ZSTD(3)),
                    instrument_id   UUID CODEC(ZSTD(3)),
                    feature_id      LowCardinality(String) CODEC(ZSTD(3)),
                    value           Float64 CODEC(ZSTD(3)),
                    insight_type    LowCardinality(String) CODEC(ZSTD(3))
                )
                ENGINE = ReplacingMergeTree
                PARTITION BY toYYYYMMDD(event_time)
                ORDER BY (pipeline_id, instrument_id, feature_id, insight_type, event_time)
                SETTINGS index_granularity = 8192;
                ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, insight: InsightClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<InsightClickhouseDTO>(TABLE_NAME).await?;
    insert.write(&insight).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, insights: &[InsightClickhouseDTO]) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<InsightClickhouseDTO>(TABLE_NAME).await?;
    for insight in insights {
        insert.write(insight).await?;
    }
    insert.end().await?;
    Ok(())
}
