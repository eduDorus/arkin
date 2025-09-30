use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, BIND_LIMIT};

const FIELD_COUNT: usize = 9;

#[derive(Debug, FromRow)]
pub struct TransferDTO {
    pub id: Uuid,
    pub transfer_group_id: Uuid,
    pub transfer_group_type: TransferGroupType,
    pub transfer_type: TransferType,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: Decimal,
    pub unit_price: Decimal,
    pub strategy_id: Option<Uuid>,
    pub instrument_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub created: OffsetDateTime,
}

impl From<Arc<Transfer>> for TransferDTO {
    fn from(transfer: Arc<Transfer>) -> Self {
        Self {
            id: transfer.id,
            transfer_group_id: transfer.transfer_group_id,
            transfer_group_type: transfer.transfer_group_type,
            transfer_type: transfer.transfer_type.clone(),
            debit_account_id: transfer.debit_account.id,
            credit_account_id: transfer.credit_account.id,
            amount: transfer.amount,
            unit_price: transfer.unit_price,
            strategy_id: transfer.strategy.as_ref().map(|s| s.id),
            instrument_id: transfer.instrument.as_ref().map(|i| i.id),
            asset_id: transfer.asset.as_ref().map(|a| a.id),
            created: transfer.created.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, transfer: TransferDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO transfers
            (
                id, 
                instance_id, 
                transfer_group_id, 
                transfer_group_type, 
                transfer_type, 
                debit_account_id, 
                credit_account_id, 
                amount, 
                unit_price, 
                strategy_id, 
                instrument_id,
                asset_id, 
                created
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        transfer.id,
        ctx.instance.id,
        transfer.transfer_group_id,
        transfer.transfer_group_type as TransferGroupType,
        transfer.transfer_type as TransferType,
        transfer.debit_account_id,
        transfer.credit_account_id,
        transfer.amount,
        transfer.unit_price,
        transfer.strategy_id,
        transfer.instrument_id,
        transfer.asset_id,
        transfer.created
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn insert_batch(ctx: &PersistenceContext, transfers: Vec<TransferDTO>) -> Result<(), PersistenceError> {
    // Build batched insert queries
    for batch in transfers.chunks(BIND_LIMIT / FIELD_COUNT) {
        // Create a query builder
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
                INSERT INTO transfers
                (
                    id, 
                    instance_id, 
                    transfer_group_id, 
                    transfer_group_type, 
                    transfer_type, 
                    debit_account_id, 
                    credit_account_id, 
                    amount, 
                    unit_price, 
                    strategy_id, 
                    instrument_id,
                    asset_id, 
                    created
                ) 
                "#,
        );

        // Push the values into the query builder
        query_builder.push_values(batch, |mut b, t| {
            b.push_bind(t.id)
                .push_bind(ctx.instance.id)
                .push_bind(t.transfer_group_id)
                .push_bind(t.transfer_group_type.clone())
                .push_bind(t.transfer_type.clone())
                .push_bind(t.debit_account_id)
                .push_bind(t.credit_account_id)
                .push_bind(t.amount)
                .push_bind(t.unit_price)
                .push_bind(t.strategy_id)
                .push_bind(t.instrument_id)
                .push_bind(t.asset_id)
                .push_bind(t.created);
        });

        // Use ON CONFLICT for the composite primary key
        // query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
        let query = query_builder.build();

        query.execute(&ctx.pg_pool).await?;
    }
    Ok(())
}
