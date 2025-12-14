use clickhouse::{sql::Identifier, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

const TABLE_NAME: &str = "audit";

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct AuditClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instance_id: Uuid,
    pub event_type: String,
    pub message: String,
}

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    ctx.ch_client
        .query(
            "
            CREATE TABLE IF NOT EXISTS ?
            (
                event_time DateTime64(3, 'UTC') CODEC(Delta, ZSTD(3)),
                instance_id UUID CODEC(ZSTD(3)),
                event_type LowCardinality(String) CODEC(ZSTD(3)),
                message String CODEC(ZSTD(3))
            )
            ENGINE = MergeTree
            PARTITION BY toYYYYMM(event_time)
            ORDER BY (event_type, instance_id, event_time)
            SETTINGS index_granularity = 8192;
            ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, audit: AuditClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<AuditClickhouseDTO>(TABLE_NAME).await?;
    insert.write(&audit).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, audits: &[AuditClickhouseDTO]) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<AuditClickhouseDTO>(TABLE_NAME).await?;
    for audit in audits {
        insert.write(audit).await?;
    }
    insert.end().await?;
    Ok(())
}
