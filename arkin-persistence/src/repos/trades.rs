use anyhow::Result;
use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

use crate::BIND_LIMIT;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "market_side", rename_all = "snake_case")]
pub enum DBMarketSide {
    Buy,
    Sell,
}

impl From<MarketSide> for DBMarketSide {
    fn from(side: MarketSide) -> Self {
        match side {
            MarketSide::Buy => Self::Buy,
            MarketSide::Sell => Self::Sell,
        }
    }
}

impl From<DBMarketSide> for MarketSide {
    fn from(side: DBMarketSide) -> Self {
        match side {
            DBMarketSide::Buy => Self::Buy,
            DBMarketSide::Sell => Self::Sell,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct DBTrade {
    pub event_time: OffsetDateTime,
    pub instrument_id: Uuid,
    pub trade_id: i64,
    pub side: DBMarketSide,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl From<Trade> for DBTrade {
    fn from(trade: Trade) -> Self {
        Self {
            event_time: trade.event_time,
            instrument_id: trade.instrument.id,
            trade_id: trade.trade_id as i64,
            side: DBMarketSide::from(trade.side),
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}

#[derive(Debug)]
pub struct TradeRepo {
    pool: PgPool,
}

impl TradeRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, trade: Trade) -> Result<()> {
        let trade = DBTrade::from(trade);
        sqlx::query!(
            r#"
            INSERT INTO trades (event_time, instrument_id, trade_id, side, price, quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (event_time, instrument_id, trade_id)
            DO NOTHING
            "#,
            trade.event_time,
            trade.instrument_id,
            trade.trade_id,
            trade.side as DBMarketSide,
            trade.price,
            trade.quantity,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, trades: Vec<Trade>) -> Result<()> {
        let db_trades = trades.into_iter().map(DBTrade::from).collect::<Vec<_>>();

        for batch in db_trades.chunks(BIND_LIMIT / 7) {
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
        debug!("Saved {} trades", db_trades.len());
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<DBTrade>> {
        let trades = sqlx::query_as!(
            DBTrade,
            r#"
            SELECT
                event_time,
                instrument_id,
                trade_id,
                side as "side:DBMarketSide",
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
