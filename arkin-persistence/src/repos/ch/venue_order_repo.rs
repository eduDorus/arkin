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

const TABLE_NAME: &str = "venue_orders";

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct VenueOrderClickhouseDTO {
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
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    #[serde(with = "custom_serde::decimal64")]
    pub price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub quantity: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub last_fill_price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub last_fill_quantity: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub last_fill_commission: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub filled_price: Decimal,
    #[serde(with = "custom_serde::decimal64")]
    pub filled_quantity: Decimal,
    #[serde(with = "clickhouse::serde::uuid")]
    pub commission_asset_id: Uuid,
    #[serde(with = "custom_serde::decimal64")]
    pub commission: Decimal,
    pub status: String,
}

impl VenueOrderClickhouseDTO {
    pub fn from_model(order: &Arc<VenueOrder>, instance_id: Uuid) -> Self {
        Self {
            event_time: order.updated.into(),
            id: order.id,
            instance_id,
            strategy_id: order.strategy.as_ref().map(|s| s.id).unwrap_or_default(),
            instrument_id: order.instrument.id,
            side: order.side.to_string(),
            order_type: order.order_type.to_string(),
            time_in_force: order.time_in_force.to_string(),
            price: order.price,
            quantity: order.quantity,
            last_fill_price: order.last_fill_price,
            last_fill_quantity: order.last_fill_quantity,
            last_fill_commission: order.last_fill_commission,
            filled_price: order.filled_price,
            filled_quantity: order.filled_quantity,
            commission_asset_id: order.commission_asset.as_ref().map(|a| a.id).unwrap_or_default(),
            commission: order.commission,
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
                side LowCardinality(String) CODEC(ZSTD(3)),
                order_type LowCardinality(String) CODEC(ZSTD(3)),
                time_in_force LowCardinality(String) CODEC(ZSTD(3)),
                price Decimal(18, 8) CODEC(ZSTD(3)),
                quantity Decimal(18, 8) CODEC(ZSTD(3)),
                last_fill_price Decimal(18, 8) CODEC(ZSTD(3)),
                last_fill_quantity Decimal(18, 8) CODEC(ZSTD(3)),
                last_fill_commission Decimal(18, 8) CODEC(ZSTD(3)),
                filled_price Decimal(18, 8) CODEC(ZSTD(3)),
                filled_quantity Decimal(18, 8) CODEC(ZSTD(3)),
                commission_asset_id UUID CODEC(ZSTD(3)),
                commission Decimal(18, 8) CODEC(ZSTD(3)),
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

pub async fn insert(ctx: &PersistenceContext, order: VenueOrderClickhouseDTO) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<VenueOrderClickhouseDTO>(TABLE_NAME).await?;
    insert.write(&order).await?;
    insert.end().await?;
    Ok(())
}

pub async fn insert_batch(
    ctx: &PersistenceContext,
    orders: &[VenueOrderClickhouseDTO],
) -> Result<(), PersistenceError> {
    let mut insert = ctx.ch_client.insert::<VenueOrderClickhouseDTO>(TABLE_NAME).await?;
    for order in orders {
        insert.write(order).await?;
    }
    insert.end().await?;
    Ok(())
}
