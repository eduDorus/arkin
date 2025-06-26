use std::fmt;
use std::sync::Arc;

use arkin_core::Insight;
use clickhouse::{sql::Identifier, Client, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::PersistenceError;

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
            event_time: insight.event_time,
            pipeline_id: insight.pipeline.as_ref().map(|p| p.id).unwrap(),
            instrument_id: insight.instrument.as_ref().map(|i| i.id).unwrap(),
            feature_id: insight.feature_id.to_string(),
            value: insight.value,
            insight_type: insight.insight_type.to_string(),
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct InsightsClickhouseRepo {
    client: Client,
    table_name: String,
}

impl fmt::Debug for InsightsClickhouseRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InsightsClickhouseRepo").finish()
    }
}

impl InsightsClickhouseRepo {
    pub fn new(client: Client) -> Self {
        let table_name = "insights";

        InsightsClickhouseRepo {
            client,
            table_name: table_name.to_string(),
        }
    }

    pub async fn create_table(&self) -> Result<(), PersistenceError> {
        self.client
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
            .bind(Identifier(&self.table_name))
            .execute()
            .await?;
        Ok(())
    }

    pub async fn insert(&self, insight: InsightClickhouseDTO) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        insert.write(&insight).await?;
        insert.end().await?;
        Ok(())
    }

    pub async fn insert_batch(&self, insights: &[InsightClickhouseDTO]) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        for insight in insights {
            insert.write(insight).await?;
        }
        insert.end().await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), PersistenceError> {
        Ok(())
    }
}
