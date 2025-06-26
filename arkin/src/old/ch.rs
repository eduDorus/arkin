use std::time::Instant;

use anyhow::Result;
use clickhouse::Client;
use clickhouse::{sql::Identifier, Row};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use time::macros::datetime;
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct InsightClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub pipeline_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub feature_id: String,
    #[serde(with = "custom_serde::decimal128")]
    pub value: Decimal,
}

#[derive(Debug, Serialize, Row)]
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

#[tokio::main]
async fn main() -> Result<()> {
    // // Show the output of a serialized FixedPoint<i64, U8>
    // let value = Decimal::new(123450, 1);

    // // Convert the Decimal to a FixedPoint<i128, U12>
    // // let value = Decimal128::from_decimal(value.mantissa(), -(value.scale() as i32))?;
    // // println!("Decimal128: {:?}", value);

    // let table_name = "insights";
    let client = Client::default()
        .with_url("http://127.0.0.1:8123")
        .with_compression(clickhouse::Compression::Lz4)
        .with_database("arkin")
        .with_user("arkin_admin")
        .with_password("test1234")
        .with_option("wait_end_of_query", "1");

    // // Create the table
    // client
    //     .query(
    //         "
    //         CREATE TABLE IF NOT EXISTS ?
    //         (
    //             event_time      DateTime64(3, 'UTC') CODEC(Delta, ZSTD(3)),
    //             pipeline_id     UUID CODEC(ZSTD(3)),
    //             instrument_id   UUID CODEC(ZSTD(3)),
    //             feature_id      LowCardinality(String) CODEC(ZSTD(3)),
    //             value           Decimal(28, 8) CODEC(ZSTD(3))
    //         )
    //         ENGINE = ReplacingMergeTree
    //         PARTITION BY toYYYYMMDD(event_time)
    //         ORDER BY (pipeline_id, instrument_id, feature_id, event_time)
    //         SETTINGS index_granularity = 8192;
    //         ",
    //     )
    //     .bind(Identifier(table_name))
    //     .execute()
    //     .await?;
    // println!("Table '{}' is ready.", table_name);

    // Create an instance of InsightClickhouseDTO
    // let dto = InsightClickhouseDTO {
    //     event_time: OffsetDateTime::now_utc(),
    //     pipeline_id: Uuid::new_v4(),
    //     instrument_id: Uuid::new_v4(),
    //     feature_id: "feature_123".to_string(),
    //     value, // Represents 123.45
    // };

    // // Insert the DTO into ClickHouse
    // let mut insert = client.insert(table_name)?;
    // insert.write(&dto).await?;
    // insert.end().await?;
    // println!("Data inserted successfully.");

    // let tick_dto = TickClickhouseDTO {
    //     event_time: OffsetDateTime::now_utc(),
    //     instrument_id: Uuid::new_v4(),
    //     tick_id: 123,
    //     bid_price: Decimal::new(123450, 1),
    //     bid_quantity: Decimal::new(123450, 1),
    //     ask_price: Decimal::new(123450, 1),
    //     ask_quantity: Decimal::new(123450, 1),
    // };

    // let table_name = "ticks";
    // let mut insert = client.insert(table_name)?;
    // insert.write(&tick_dto).await?;
    // insert.end().await?;
    // println!("Data inserted successfully.");

    // let trade_dto = TradeClickhouseDTO {
    //     event_time: OffsetDateTime::now_utc(),
    //     instrument_id: Uuid::new_v4(),
    //     trade_id: 123,
    //     side: 1,
    //     price: Decimal::new(123450, 1),
    //     quantity: Decimal::new(123450, 1),
    // };

    // let table_name = "trades";
    // let mut insert = client.insert(table_name)?;
    // insert.write(&trade_dto).await?;
    // insert.end().await?;
    // println!("Data inserted successfully.");

    // Let's read data from the trades table
    let timer = Instant::now();
    let instrument_ids = vec![
        Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").unwrap(),
        Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").unwrap(),
    ];
    let from = datetime!(2024-01-01 00:00:00).assume_utc();
    let till = datetime!(2024-01-07 00:00:00).assume_utc();
    let table_name = "trades";
    let cursor = client
        .query(
            r#"
            SELECT 
              ?fields 
            FROM ? FINAL
            WHERE 
              instrument_id IN (?)
              AND event_time BETWEEN ? AND ? 
            ORDER BY 
              event_time ASC"#,
        )
        .bind(Identifier(table_name))
        .bind(instrument_ids)
        .bind(from.unix_timestamp())
        .bind(till.unix_timestamp())
        .fetch_all::<TradeClickhouseDTO>()
        .await?;

    println!("Total rows: {}", cursor.len());
    // let mut count = 0;
    // while let Some(_row) = cursor.next().await? {
    //     count += 1;
    // }
    // println!("Total rows: {}", count);
    println!("Time elapsed: {:?}", timer.elapsed());
    Ok(())
}
