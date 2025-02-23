use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 9;

#[derive(Debug, FromRow)]
pub struct TransferDTO {
    pub event_time: OffsetDateTime,
    pub transfer_group_id: Uuid,
    pub portfolio_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub instrument_id: Option<Uuid>,
    pub transfer_type: TransferType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub total_value: Decimal,
}

impl From<Arc<Transfer>> for TransferDTO {
    fn from(transfer: Arc<Transfer>) -> Self {
        Self {
            event_time: transfer.event_time,
            transfer_group_id: transfer.transfer_group_id,
            portfolio_id: transfer.portfolio.id,
            asset_id: transfer.asset.as_ref().map(|a| a.id),
            instrument_id: transfer.instrument.as_ref().map(|i| i.id),
            transfer_type: transfer.transfer_type.clone(),
            price: transfer.price,
            quantity: transfer.quantity,
            total_value: transfer.total_value,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct TransferRepo {
    pool: PgPool,
}

impl TransferRepo {
    pub async fn insert(&self, transfer: TransferDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO transfers
            (
                event_time, 
                transfer_group_id, 
                portfolio_id, 
                asset_id, 
                instrument_id, 
                transfer_type, 
                price, 
                quantity, 
                total_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            transfer.event_time,
            transfer.transfer_group_id,
            transfer.portfolio_id,
            transfer.asset_id,
            transfer.instrument_id,
            transfer.transfer_type as TransferType,
            transfer.price,
            transfer.quantity,
            transfer.total_value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, transfers: Vec<TransferDTO>) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in transfers.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO transfers
                (
                    event_time, 
                    transfer_group_id, 
                    portfolio_id, 
                    asset_id, 
                    instrument_id, 
                    transfer_type, 
                    price, 
                    quantity, 
                    total_value
                ) 
                "#,
            );

            // Push the values into the query builder
            query_builder.push_values(batch, |mut b, tick| {
                b.push_bind(tick.event_time)
                    .push_bind(tick.transfer_group_id)
                    .push_bind(tick.portfolio_id)
                    .push_bind(tick.asset_id)
                    .push_bind(tick.instrument_id)
                    .push_bind(tick.transfer_type)
                    .push_bind(tick.price)
                    .push_bind(tick.quantity)
                    .push_bind(tick.total_value);
            });

            // Use ON CONFLICT for the composite primary key
            // query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        Ok(())
    }
}
