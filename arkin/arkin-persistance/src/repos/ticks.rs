use arkin_core::{prelude::Tick, Price, Quantity};
use futures_util::{stream, StreamExt};
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;
use anyhow::Result;

use crate::{BIND_LIMIT, MAX_CONCURRENT_QUERIES};

#[derive(Debug, FromRow)]
pub struct DBTick {
    pub instrument_id: Uuid,
    pub event_time: OffsetDateTime,
    pub tick_id: i64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl From<Tick> for DBTick {
    fn from(tick: Tick) -> Self {
        Self {
            instrument_id: tick.instrument.id,
            event_time: tick.event_time,
            tick_id: tick.tick_id as i64,
            bid_price: tick.bid_price,
            bid_quantity: tick.bid_quantity,
            ask_price: tick.ask_price,
            ask_quantity: tick.ask_quantity,
        }
    }
}

pub struct TickRepo {
    pool: PgPool,
}

impl TickRepo {
    pub fn new(pool: PgPool) -> Self {
        Self {pool}
    }

    pub async fn insert(&self, tick: Tick) -> Result<()> {
        let tick = DBTick::from(tick);
        sqlx::query!(
            r#"
            INSERT INTO ticks (instrument_id, event_time, tick_id, bid_price, bid_quantity, ask_price, ask_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            tick.instrument_id,
            tick.event_time,
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
        let ticks = ticks.into_iter().map(DBTick::from).collect::<Vec<_>>();

        let queries = ticks
            .chunks(BIND_LIMIT / 7)
            .map(|batch| {
                // Create a query builder
                let mut query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO ticks (instrument_id, event_time, tick_id, bid_price, bid_quantity, ask_price, ask_quantity) ",
                );

                // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
                query_builder.push_values(batch, |mut b, trade| {
                    // If you wanted to bind these by-reference instead of by-value,
                    // you'd need an iterator that yields references that live as long as `query_builder`,
                    // e.g. collect it to a `Vec` first.
                    b.push_bind(trade.instrument_id)
                        .push_bind(trade.event_time)
                        .push_bind(trade.tick_id)
                        .push_bind(trade.bid_price)
                        .push_bind(trade.bid_quantity)
                        .push_bind(trade.ask_price)
                        .push_bind(trade.ask_quantity); 
                });

                query_builder
            })
            .collect::<Vec<_>>();

        let query_stream = stream::iter(queries.into_iter().map(|mut query| {
            let db_pool = self.pool.clone();
            async move { query.build().execute(&db_pool).await }
        }));

        let results = query_stream.buffered(MAX_CONCURRENT_QUERIES).collect::<Vec<_>>().await;

        for result in results {
            match result {
                Ok(_) => { /* Success */ }
                Err(e) => {
                    error!("Error executing query: {}", e);
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        start: &OffsetDateTime,
        end: &OffsetDateTime,
    ) -> Result<Vec<DBTick>> {
        let ticks = sqlx::query_as!(
            DBTick,
            r#"
            SELECT * FROM ticks
            WHERE event_time >= $1 AND event_time <= $2 AND instrument_id = ANY($3)
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
