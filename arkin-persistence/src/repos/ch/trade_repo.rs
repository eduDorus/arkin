use std::sync::Arc;

use clickhouse::{query::RowCursor, sql::Identifier, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, PersistenceError};

const TABLE_NAME: &str = "trades";

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct TradeClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub trade_id: u64,
    pub side: i8,
    #[serde(with = "custom_serde::decimal64")]
    pub price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub quantity: Decimal, // Negative for sell, positive for buy
}

impl From<Arc<AggTrade>> for TradeClickhouseDTO {
    fn from(trade: Arc<AggTrade>) -> Self {
        Self {
            event_time: trade.event_time.into(),
            instrument_id: trade.instrument.id,
            trade_id: trade.trade_id,
            side: trade.side.into(),
            price: trade.price,
            quantity: trade.quantity,
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
                trade_id       UInt64 CODEC(Delta, ZSTD(3)),
                side           Int8 CODEC(ZSTD(3)),
                price          Decimal(18, 8) CODEC(GCD, ZSTD(3)),
                quantity       Decimal(18, 8) CODEC(GCD, ZSTD(3))
            )
            ENGINE = ReplacingMergeTree
            PARTITION BY toYYYYMMDD(event_time)
            ORDER BY (instrument_id, event_time, trade_id)
            SETTINGS index_granularity = 8192;
            ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, trade: TradeClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert(TABLE_NAME)?;
    insert.write(&trade).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, trades: &[TradeClickhouseDTO]) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert(TABLE_NAME)?;
    for trade in trades {
        insert.write(trade).await?;
    }
    insert.end().await?;
    Ok(())
}

pub async fn read_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    from: UtcDateTime,
    till: UtcDateTime,
) -> Result<Vec<TradeClickhouseDTO>, PersistenceError> {
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
                  event_time ASC"#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(from.unix_timestamp())
        .bind(till.unix_timestamp())
        .bind(instrument_ids)
        .fetch_all::<TradeClickhouseDTO>()
        .await?;
    Ok(cursor)
}

pub async fn stream_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    from: UtcDateTime,
    till: UtcDateTime,
) -> Result<RowCursor<TradeClickhouseDTO>, PersistenceError> {
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
                  event_time, trade_id ASC"#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(from.unix_timestamp())
        .bind(till.unix_timestamp())
        .bind(instrument_ids)
        .fetch::<TradeClickhouseDTO>()?;
    Ok(cursor)
}

pub async fn fetch_batch(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    day_start: UtcDateTime,
    day_end: UtcDateTime,
) -> Result<Vec<TradeClickhouseDTO>, PersistenceError> {
    let rows = ctx
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
                event_time, trade_id ASC
            "#,
        )
        .bind(Identifier(TABLE_NAME))
        .bind(day_start.unix_timestamp())
        .bind(day_end.unix_timestamp())
        .bind(instrument_ids)
        .fetch_all::<TradeClickhouseDTO>()
        .await?;
    Ok(rows)
}
