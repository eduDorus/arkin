use anyhow::Result;
use arkin_core::{prelude::Tick, Price, Quantity};
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

use crate::BIND_LIMIT;

#[derive(Debug, FromRow)]
pub struct DBTick {
    pub event_time: OffsetDateTime,
    pub instrument_id: Uuid,
    pub tick_id: i64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl From<Tick> for DBTick {
    fn from(tick: Tick) -> Self {
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

#[derive(Debug)]
pub struct TickRepo {
    pool: PgPool,
}

impl TickRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, tick: Tick) -> Result<()> {
        let tick = DBTick::from(tick);
        sqlx::query!(
            r#"
            INSERT INTO ticks (event_time, instrument_id, tick_id, bid_price, bid_quantity, ask_price, ask_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (event_time, instrument_id, tick_id)
            DO NOTHING
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

    pub async fn insert_batch(&self, ticks: Vec<Tick>) -> Result<()> {
        // Convert Tick to DBTick and prepare for insertion
        let db_ticks = ticks.into_iter().map(DBTick::from).collect::<Vec<_>>();

        // Build batched insert queries
        for batch in db_ticks.chunks(BIND_LIMIT / 7) {
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
        info!("Saved ticks");
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DBTick>> {
        let ticks = sqlx::query_as!(
            DBTick,
            r#"
            SELECT * FROM ticks
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
