use arkin_core::{prelude::Tick, Price, Quantity};
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

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

impl DBTick {
    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO ticks (instrument_id, event_time, tick_id, bid_price, bid_quantity, ask_price, ask_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            self.instrument_id,
            self.event_time,
            self.tick_id,
            self.bid_price,
            self.bid_quantity,
            self.ask_price,
            self.ask_quantity,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
