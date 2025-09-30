use std::sync::Arc;

use clickhouse::{query::RowCursor, sql::Identifier, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};
use uuid::Uuid;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

const TABLE_NAME: &str = "ticks";

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct TickClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub tick_id: u64,
    #[serde(with = "custom_serde::decimal64")]
    pub bid_price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub bid_quantity: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub ask_price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub ask_quantity: Decimal,
}

impl From<Arc<Tick>> for TickClickhouseDTO {
    fn from(tick: Arc<Tick>) -> Self {
        Self {
            event_time: tick.event_time.into(),
            instrument_id: tick.instrument.id,
            tick_id: tick.tick_id,
            bid_price: tick.bid_price,
            bid_quantity: tick.bid_quantity,
            ask_price: tick.ask_price,
            ask_quantity: tick.ask_quantity,
        }
    }
}

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    ctx.ch_client
        .query(
            "
          CREATE TABLE IF NOT EXISTS ?
          (
              event_time     DateTime64(3, 'UTC') CODEC(Delta, ZSTD(3)),
              instrument_id  UUID CODEC(ZSTD(3)),
              tick_id        UInt64 CODEC(Delta, ZSTD(3)),
              bid_price      Decimal(18, 8) CODEC(GCD, ZSTD(3)),
              bid_quantity   Decimal(18, 8) CODEC(GCD, ZSTD(3)),
              ask_price      Decimal(18, 8) CODEC(GCD, ZSTD(3)),
              ask_quantity   Decimal(18, 8) CODEC(GCD, ZSTD(3))
          )
          ENGINE = ReplacingMergeTree
          PARTITION BY toYYYYMMDD(event_time)
          ORDER BY (instrument_id, event_time, tick_id)
          SETTINGS index_granularity = 8192;
          ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, tick: TickClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert(TABLE_NAME)?;
    insert.write(&tick).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, ticks: &[TickClickhouseDTO]) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert(TABLE_NAME)?;
    for tick in ticks {
        insert.write(tick).await?;
    }
    insert.end().await?;
    Ok(())
}

pub async fn read_last(
    ctx: &PersistenceContext,
    instrument_id: &Uuid,
) -> Result<Option<TickClickhouseDTO>, PersistenceError> {
    let res = ctx
        .ch_client
        .query(
            r#"
              SELECT
                event_time, instrument_id, tick_id, bid_price, bid_quantity, ask_price, ask_quantity
              FROM
                ? 
              WHERE
                instrument_id = ?
              ORDER BY
                event_time DESC
              LIMIT 1
              "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(instrument_id)
        .fetch_optional::<TickClickhouseDTO>()
        .await?;
    Ok(res)
}

pub async fn read_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    from: UtcDateTime,
    till: UtcDateTime,
) -> Result<Vec<TickClickhouseDTO>, PersistenceError> {
    let cursor = ctx
        .ch_client
        .query(
            r#"
              SELECT 
                ?fields 
              FROM 
                ? FINAL
              WHERE 
                instrument_id IN (?)
                AND event_time BETWEEN ? AND ? 
              ORDER BY 
                event_time ASC
              "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(instrument_ids)
        .bind(from.unix_timestamp())
        .bind(till.unix_timestamp())
        .fetch_all::<TickClickhouseDTO>()
        .await?;
    Ok(cursor)
}

pub async fn stream_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    from: UtcDateTime,
    till: UtcDateTime,
) -> Result<RowCursor<TickClickhouseDTO>, PersistenceError> {
    let cursor = ctx
        .ch_client
        .query(
            r#"
              SELECT 
                ?fields 
              FROM 
                ? FINAL
              WHERE 
                event_time BETWEEN ? AND ? 
                AND instrument_id IN (?)
              ORDER BY 
                event_time, tick_id ASC
              "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(from.unix_timestamp())
        .bind(till.unix_timestamp())
        .bind(instrument_ids)
        .fetch::<TickClickhouseDTO>()?;
    Ok(cursor)
}

pub async fn fetch_batch(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    day_start: UtcDateTime,
    day_end: UtcDateTime,
) -> Result<Vec<TickClickhouseDTO>, PersistenceError> {
    let rows = ctx
            .ch_client
            .query(
                r#"
              SELECT
                  arrayElement(arraySort((x, y) -> y, groupArray(t.event_time), groupArray(t.event_time)), -1) AS event_time,
                  t.instrument_id,
                  arrayElement(arraySort((x, y) -> y, groupArray(t.tick_id), groupArray(t.event_time)), -1) AS tick_id,
                  arrayElement(arraySort((x, y) -> y, groupArray(t.bid_price), groupArray(t.event_time)), -1) AS bid_price,
                  arrayElement(arraySort((x, y) -> y, groupArray(t.bid_quantity), groupArray(t.event_time)), -1) AS bid_quantity,
                  arrayElement(arraySort((x, y) -> y, groupArray(t.ask_price), groupArray(t.event_time)), -1) AS ask_price,
                  arrayElement(arraySort((x, y) -> y, groupArray(t.ask_quantity), groupArray(t.event_time)), -1) AS ask_quantity
              FROM 
                  ? t FINAL
              WHERE 
                  t.event_time BETWEEN ? AND ?
                  AND t.instrument_id IN (?)
              GROUP BY 
                  t.instrument_id, 
                  toStartOfInterval(t.event_time, INTERVAL 1 SECONDS)
              ORDER BY 
                  event_time
              "#,
            )
            .bind(Identifier(TABLE_NAME))
            .bind(day_start.unix_timestamp())
            .bind(day_end.unix_timestamp())
            .bind(instrument_ids)
            .fetch_all::<TickClickhouseDTO>()
            .await?;
    Ok(rows)
}
