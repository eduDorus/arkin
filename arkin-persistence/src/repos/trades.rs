use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 6;

#[derive(Debug, FromRow)]
pub struct TradeDTO {
    pub event_time: OffsetDateTime,
    pub instrument_id: Uuid,
    pub trade_id: i64,
    pub side: MarketSide,
    pub price: Decimal,
    pub quantity: Decimal, // Negative for sell, positive for buy
}

impl From<Arc<Trade>> for TradeDTO {
    fn from(trade: Arc<Trade>) -> Self {
        Self {
            event_time: trade.event_time,
            instrument_id: trade.instrument.id,
            trade_id: trade.trade_id as i64,
            side: trade.side,
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct TradeRepo {
    pool: PgPool,
}

impl TradeRepo {
    pub async fn insert(&self, trade: TradeDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO trades 
            (
                event_time, 
                instrument_id, 
                trade_id, 
                side, 
                price, 
                quantity
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (event_time, instrument_id, trade_id) DO NOTHING
            "#,
            trade.event_time,
            trade.instrument_id,
            trade.trade_id,
            trade.side as MarketSide,
            trade.price,
            trade.quantity,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, trades: Vec<TradeDTO>) -> Result<(), PersistenceError> {
        for batch in trades.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO trades (event_time, instrument_id, trade_id, side, price, quantity) ",
            );

            // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
            query_builder.push_values(batch, |mut b, trade| {
                // If you wanted to bind these by-reference instead of by-value,
                // you'd need an iterator that yields references that live as long as `query_builder`,
                // e.g. collect it to a `Vec` first.
                b.push_bind(trade.event_time)
                    .push_bind(trade.instrument_id)
                    .push_bind(trade.trade_id)
                    .push_bind(trade.side.clone())
                    .push_bind(trade.price)
                    .push_bind(trade.quantity);
            });

            // Use ON CONFLICT for the composite primary key
            query_builder.push(" ON CONFLICT (instrument_id, trade_id, event_time) DO NOTHING");

            let query = query_builder.build();
            query.execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<TradeDTO>, PersistenceError> {
        let trades = sqlx::query_as!(
            TradeDTO,
            r#"
            SELECT
                event_time,
                instrument_id,
                trade_id,
                side as "side:MarketSide",
                price,
                quantity
            FROM trades
            WHERE instrument_id = ANY($1) AND event_time >= $2 AND event_time < $3
            ORDER BY event_time ASC
            "#,
            instrument_ids,
            from,
            to,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(trades)
    }
}
