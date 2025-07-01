use std::{fmt, sync::Arc};

use clickhouse::{query::RowCursor, sql::Identifier, Client, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::PersistenceError;

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
            event_time: tick.event_time,
            instrument_id: tick.instrument.id,
            tick_id: tick.tick_id,
            bid_price: tick.bid_price,
            bid_quantity: tick.bid_quantity,
            ask_price: tick.ask_price,
            ask_quantity: tick.ask_quantity,
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct TickClickhouseRepo {
    client: Client,
    table_name: String,
}

impl fmt::Debug for TickClickhouseRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TickClickhouseRepo").finish()
    }
}

impl TickClickhouseRepo {
    pub fn new(client: Client) -> Self {
        let table_name = "ticks";

        TickClickhouseRepo {
            client,
            table_name: table_name.to_string(),
        }
    }

    pub async fn close(&self) -> Result<(), PersistenceError> {
        Ok(())
    }

    pub async fn create_table(&self) -> Result<(), PersistenceError> {
        self.client
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
            .bind(Identifier(&self.table_name))
            .execute()
            .await?;
        Ok(())
    }

    pub async fn insert(&self, tick: TickClickhouseDTO) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        insert.write(&tick).await?;
        insert.end().await?;
        Ok(())
    }

    pub async fn insert_batch(&self, ticks: &[TickClickhouseDTO]) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        for tick in ticks {
            insert.write(tick).await?;
        }
        insert.end().await?;
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        till: OffsetDateTime,
    ) -> Result<Vec<TickClickhouseDTO>, PersistenceError> {
        let cursor = self
            .client
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
            .bind(Identifier(&self.table_name))
            .bind(instrument_ids)
            .bind(from.unix_timestamp())
            .bind(till.unix_timestamp())
            .fetch_all::<TickClickhouseDTO>()
            .await?;
        Ok(cursor)
    }

    pub async fn stream_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        till: OffsetDateTime,
    ) -> Result<RowCursor<TickClickhouseDTO>, PersistenceError> {
        let cursor = self
            .client
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
            .bind(Identifier(&self.table_name))
            .bind(from.unix_timestamp())
            .bind(till.unix_timestamp())
            .bind(instrument_ids)
            .fetch::<TickClickhouseDTO>()?;
        Ok(cursor)
    }

    pub async fn fetch_batch(
        &self,
        instrument_ids: &[Uuid],
        day_start: OffsetDateTime,
        day_end: OffsetDateTime,
    ) -> Result<Vec<TickClickhouseDTO>, PersistenceError> {
        let rows = self
            .client
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
            .bind(Identifier(&self.table_name))
            .bind(day_start.unix_timestamp())
            .bind(day_end.unix_timestamp())
            .bind(instrument_ids)
            .fetch_all::<TickClickhouseDTO>()
            .await?;
        Ok(rows)
    }
}
