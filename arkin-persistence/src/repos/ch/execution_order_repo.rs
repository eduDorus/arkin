use std::sync::Arc;

use clickhouse::{sql::Identifier, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::prelude::*;
use arkin_core::utils::custom_serde;
use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

const TABLE_NAME: &str = "execution_orders";

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ExecutionOrderClickhouseDTO {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub event_time: OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instance_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub strategy_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub instrument_id: Uuid,
    pub order_type: String,
    pub side: String,
    #[serde(with = "custom_serde::decimal64")]
    pub price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub quantity: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub fill_price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub filled_quantity: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub total_commission: Decimal,
    pub status: String,
}

impl ExecutionOrderClickhouseDTO {
    pub fn from_model(order: &Arc<ExecutionOrder>, instance_id: Uuid) -> Self {
        Self {
            event_time: order.updated.into(),
            id: order.id,
            instance_id,
            strategy_id: order.strategy.as_ref().map(|s| s.id).unwrap_or_default(),
            instrument_id: order.instrument.id,
            order_type: order.exec_strategy_type.to_string(),
            side: order.side.to_string(),
            price: order.price,
            quantity: order.quantity,
            fill_price: order.fill_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.total_commission,
            status: order.status.to_string(),
        }
    }
}

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    ctx.ch_client
        .query(
            "
            CREATE TABLE IF NOT EXISTS ?
            (
                event_time DateTime64(3, 'UTC') CODEC(Delta, ZSTD(3)),
                id UUID CODEC(ZSTD(3)),
                instance_id UUID CODEC(ZSTD(3)),
                strategy_id UUID CODEC(ZSTD(3)),
                instrument_id UUID CODEC(ZSTD(3)),
                order_type LowCardinality(String) CODEC(ZSTD(3)),
                side LowCardinality(String) CODEC(ZSTD(3)),
                price Decimal(18, 8) CODEC(ZSTD(3)),
                quantity Decimal(18, 8) CODEC(ZSTD(3)),
                fill_price Decimal(18, 8) CODEC(ZSTD(3)),
                filled_quantity Decimal(18, 8) CODEC(ZSTD(3)),
                total_commission Decimal(18, 8) CODEC(ZSTD(3)),
                status LowCardinality(String) CODEC(ZSTD(3))
            )
            ENGINE = ReplacingMergeTree
            PARTITION BY toYYYYMM(event_time)
            ORDER BY (status, id, event_time)
            SETTINGS index_granularity = 8192;
            ",
        )
        .bind(Identifier(TABLE_NAME))
        .execute()
        .await?;
    Ok(())
}

pub async fn insert(ctx: &PersistenceContext, order: ExecutionOrderClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<ExecutionOrderClickhouseDTO>(TABLE_NAME).await?;
    insert.write(&order).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(
    ctx: &PersistenceContext,
    orders: &[ExecutionOrderClickhouseDTO],
) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<ExecutionOrderClickhouseDTO>(TABLE_NAME).await?;
    for order in orders {
        insert.write(order).await?;
    }
    insert.end().await?;
    Ok(())
}
