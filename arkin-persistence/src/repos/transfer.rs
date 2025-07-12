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
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub transfer_group_id: Uuid,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: Decimal,
    pub unit_price: Decimal,
    pub transfer_type: TransferType,
    pub strategy_id: Option<Uuid>,
    pub instrument_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
}

impl From<Arc<Transfer>> for TransferDTO {
    fn from(transfer: Arc<Transfer>) -> Self {
        Self {
            id: transfer.id,
            event_time: transfer.event_time.into(),
            transfer_group_id: transfer.transfer_group_id,
            debit_account_id: transfer.debit_account.id,
            credit_account_id: transfer.credit_account.id,
            amount: transfer.amount,
            unit_price: transfer.unit_price,
            transfer_type: transfer.transfer_type.clone(),
            strategy_id: transfer.strategy.as_ref().map(|s| s.id),
            instrument_id: transfer.instrument.as_ref().map(|i| i.id),
            asset_id: transfer.asset.as_ref().map(|a| a.id),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct TransferRepo {
    pool: PgPool,
    instance: Arc<Instance>,
}

impl TransferRepo {
    pub async fn insert(&self, transfer: TransferDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO transfers
            (
                id, 
                event_time, 
                instance_id, 
                transfer_group_id, 
                debit_account_id, 
                credit_account_id, 
                asset_id, 
                amount, 
                unit_price, 
                transfer_type, 
                strategy_id, 
                instrument_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            transfer.id,
            transfer.event_time,
            self.instance.id,
            transfer.transfer_group_id,
            transfer.debit_account_id,
            transfer.credit_account_id,
            transfer.asset_id,
            transfer.amount,
            transfer.unit_price,
            transfer.transfer_type as TransferType,
            transfer.strategy_id,
            transfer.instrument_id,
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
                    id, 
                    event_time, 
                    instance_id, 
                    transfer_group_id, 
                    debit_account_id, 
                    credit_account_id, 
                    asset_id, 
                    amount, 
                    unit_price, 
                    transfer_type, 
                    strategy_id, 
                    instrument_id
                ) 
                "#,
            );

            // Push the values into the query builder
            query_builder.push_values(batch, |mut b, t| {
                b.push_bind(t.id)
                    .push_bind(t.event_time)
                    .push_bind(self.instance.id)
                    .push_bind(t.transfer_group_id)
                    .push_bind(t.debit_account_id)
                    .push_bind(t.credit_account_id)
                    .push_bind(t.asset_id)
                    .push_bind(t.amount)
                    .push_bind(t.unit_price)
                    .push_bind(t.transfer_type.clone())
                    .push_bind(t.strategy_id)
                    .push_bind(t.instrument_id);
            });

            // Use ON CONFLICT for the composite primary key
            // query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        Ok(())
    }
}
