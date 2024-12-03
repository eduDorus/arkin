use std::sync::Arc;

use derive_builder::Builder;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 7;

#[derive(Debug, FromRow)]
pub struct TickDTO {
    pub event_time: OffsetDateTime,
    pub instrument_id: Uuid,
    pub tick_id: i64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl From<Arc<Tick>> for TickDTO {
    fn from(tick: Arc<Tick>) -> Self {
        Self {
            event_time: tick.event_time,
            instrument_id: tick.instrument.id,
            tick_id: tick.tick_id as i64,
            bid_price: tick.bid_price,
            bid_quantity: tick.bid_quantity,
            ask_price: tick.ask_price,
            ask_quantity: tick.ask_quantity,
        }
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct TickRepo {
    pool: PgPool,
}

impl TickRepo {
    pub async fn insert(&self, tick: TickDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO ticks 
            (
                event_time, 
                instrument_id, 
                tick_id, 
                bid_price, 
                bid_quantity, 
                ask_price, 
                ask_quantity
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (event_time, instrument_id, tick_id) DO NOTHING
            "#,
            tick.event_time,
            tick.instrument_id,
            tick.tick_id,
            tick.bid_price,
            tick.bid_quantity,
            tick.ask_price,
            tick.ask_quantity,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, ticks: Vec<TickDTO>) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in ticks.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO ticks (event_time, instrument_id, tick_id, bid_price, bid_quantity, ask_price, ask_quantity) ",
                );

            // Push the values into the query builder
            query_builder.push_values(batch, |mut b, tick| {
                b.push_bind(tick.event_time)
                    .push_bind(tick.instrument_id)
                    .push_bind(tick.tick_id)
                    .push_bind(tick.bid_price)
                    .push_bind(tick.bid_quantity)
                    .push_bind(tick.ask_price)
                    .push_bind(tick.ask_quantity);
            });

            // Use ON CONFLICT for the composite primary key
            query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn read_tick(
        &self,
        event_time: OffsetDateTime,
        instrument_id: Uuid,
    ) -> Result<Option<TickDTO>, PersistenceError> {
        let tick = sqlx::query_as!(
            TickDTO,
            r#"
            SELECT  
                event_time, 
                instrument_id, 
                tick_id, 
                bid_price, 
                bid_quantity, 
                ask_price, 
                ask_quantity
            FROM ticks
            WHERE event_time < $1 AND instrument_id = $2
            ORDER BY event_time DESC
            "#,
            event_time,
            instrument_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(tick)
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<TickDTO>, PersistenceError> {
        let ticks = sqlx::query_as!(
            TickDTO,
            r#"
            SELECT 
                event_time, 
                instrument_id, 
                tick_id, 
                bid_price, 
                bid_quantity, 
                ask_price, 
                ask_quantity
            FROM ticks
            WHERE instrument_id = ANY($3) AND event_time >= $1 AND event_time < $2
            ORDER BY event_time ASC
            "#,
            start,
            end,
            instrument_ids,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ticks)
    }
}
