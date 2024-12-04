use std::sync::Arc;

use arkin_core::prelude::*;
use rust_decimal::Decimal;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 8;

#[derive(FromRow)]
pub struct AllocationDTO {
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub group_id: Uuid,
    pub portfolio_id: Uuid,
    pub strategy_id: Uuid,
    pub instrument_id: Uuid,
    pub signal_id: Uuid,
    pub weight: Decimal,
}

impl From<Arc<Allocation>> for AllocationDTO {
    fn from(allocation: Arc<Allocation>) -> Self {
        Self {
            id: allocation.id,
            event_time: allocation.event_time,
            group_id: allocation.group_id,
            portfolio_id: allocation.portfolio.id,
            strategy_id: allocation.strategy.id,
            instrument_id: allocation.instrument.id,
            signal_id: allocation.signal.id,
            weight: allocation.weight,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct AllocationRepo {
    pool: PgPool,
}

impl AllocationRepo {
    pub async fn insert(&self, allocation: AllocationDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO allocations 
            (
                id, 
                event_time,
                group_id, 
                portfolio_id,
                strategy_id,
                instrument_id,
                signal_id,
                weight
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            allocation.id,
            allocation.event_time,
            allocation.group_id,
            allocation.portfolio_id,
            allocation.strategy_id,
            allocation.instrument_id,
            allocation.signal_id,
            allocation.weight
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, allocations: Vec<AllocationDTO>) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in allocations.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO allocations 
                (
                    id, 
                    event_time,
                    group_id, 
                    portfolio_id,
                    strategy_id,
                    instrument_id,
                    signal_id,
                    weight
                )
                "#,
            );

            // Push the values into the query builder
            query_builder.push_values(batch, |mut b, allocation| {
                b.push_bind(allocation.id)
                    .push_bind(allocation.event_time)
                    .push_bind(allocation.group_id)
                    .push_bind(allocation.portfolio_id)
                    .push_bind(allocation.strategy_id)
                    .push_bind(allocation.instrument_id)
                    .push_bind(allocation.signal_id)
                    .push_bind(allocation.weight);
            });

            // Use ON CONFLICT for the composite primary key
            // query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        Ok(())
    }
}
