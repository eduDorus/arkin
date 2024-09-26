use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct DBTrade {
    pub instrument_id: Uuid,
    pub event_time: OffsetDateTime,
    pub trade_id: i64,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl From<Trade> for DBTrade {
    fn from(trade: Trade) -> Self {
        Self {
            instrument_id: trade.instrument.id,
            event_time: trade.event_time,
            trade_id: trade.trade_id as i64,
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}

impl DBTrade {
    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO trades (instrument_id,  event_time, trade_id, price, quantity)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.instrument_id,
            self.event_time,
            self.trade_id,
            self.price,
            self.quantity,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
