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

impl From<Arc<Trade>> for TradeClickhouseDTO {
    fn from(trade: Arc<Trade>) -> Self {
        Self {
            event_time: trade.event_time,
            instrument_id: trade.instrument.id,
            trade_id: trade.trade_id,
            side: trade.side.into(),
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct TradeClickhouseRepo {
    client: Client,
    table_name: String,
}

impl fmt::Debug for TradeClickhouseRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradeClickhouseRepo").finish()
    }
}

impl TradeClickhouseRepo {
    pub fn new() -> Self {
        let table_name = "trades";
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_compression(clickhouse::Compression::Lz4)
            .with_database("arkin")
            .with_user("arkin_admin")
            .with_password("test1234")
            .with_option("wait_end_of_query", "1");

        TradeClickhouseRepo {
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
            .bind(Identifier(&self.table_name))
            .execute()
            .await?;
        Ok(())
    }

    pub async fn insert(&self, trade: TradeClickhouseDTO) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        insert.write(&trade).await?;
        insert.end().await?;
        Ok(())
    }

    pub async fn insert_batch(&self, trades: Vec<TradeClickhouseDTO>) -> Result<(), PersistenceError> {
        let mut insert = self.client.insert(&self.table_name)?;
        for trade in trades {
            insert.write(&trade).await?;
        }
        insert.end().await?;
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        till: OffsetDateTime,
    ) -> Result<Vec<TradeClickhouseDTO>, PersistenceError> {
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
                  event_time ASC"#,
            )
            .bind(Identifier(&self.table_name))
            .bind(from.unix_timestamp())
            .bind(till.unix_timestamp())
            .bind(instrument_ids)
            .fetch_all::<TradeClickhouseDTO>()
            .await?;
        Ok(cursor)
    }

    pub async fn stream_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        till: OffsetDateTime,
    ) -> Result<RowCursor<TradeClickhouseDTO>, PersistenceError> {
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
                  event_time, trade_id ASC"#,
            )
            .bind(Identifier(&self.table_name))
            .bind(from.unix_timestamp())
            .bind(till.unix_timestamp())
            .bind(instrument_ids)
            .fetch::<TradeClickhouseDTO>()?;
        Ok(cursor)
    }

    pub async fn fetch_batch(
        &self,
        instrument_ids: &[Uuid],
        day_start: OffsetDateTime,
        day_end: OffsetDateTime,
    ) -> Result<Vec<TradeClickhouseDTO>, PersistenceError> {
        let rows = self
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
                event_time, trade_id ASC
            "#,
            )
            .bind(Identifier(&self.table_name))
            .bind(day_start.unix_timestamp())
            .bind(day_end.unix_timestamp())
            .bind(instrument_ids)
            .fetch_all::<TradeClickhouseDTO>()
            .await?;
        Ok(rows)
    }
}
