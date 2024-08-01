use super::DBManager;
use crate::models::Fill;
use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
struct FillRow {
    received_time: OffsetDateTime,
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    order_id: Option<i64>,
    strategy_id: String,
    price: Decimal,
    quantity: Decimal,
    commission: Decimal,
}

impl From<Fill> for FillRow {
    fn from(fill: Fill) -> Self {
        Self {
            received_time: fill.received_time,
            event_time: fill.event_time,
            instrument_type: fill.instrument.instrument_type().to_string(),
            venue: fill.instrument.venue().to_string(),
            base: fill.instrument.base().to_string(),
            quote: fill.instrument.quote().to_string(),
            maturity: fill.instrument.maturity().map(|m| m.value()),
            strike: fill.instrument.strike().map(|s| s.value()),
            option_type: fill.instrument.option_type().map(|ot| ot.to_string()),
            order_id: fill.order_id.map(|o| o as i64),
            strategy_id: fill.strategy_id,
            price: fill.price.value(),
            quantity: fill.quantity.value(),
            commission: fill.commission.value(),
        }
    }
}

impl DBManager {
    pub async fn insert_fill(&self, fill: Fill) -> Result<()> {
        let fill = FillRow::from(fill);
        sqlx::query!(
            r#"
            INSERT INTO fills (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, order_id, strategy_id, price, quantity, commission)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            fill.received_time,
            fill.event_time,
            fill.instrument_type,
            fill.venue,
            fill.base,
            fill.quote,
            fill.maturity,
            fill.strike,
            fill.option_type,
            fill.order_id,
            fill.strategy_id,
            fill.price,
            fill.quantity,
            fill.commission,
        )
        .execute(&self.pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::*;
    use time::OffsetDateTime;

    use super::*;
    use crate::{
        config,
        models::{Instrument, Venue},
    };

    #[tokio::test]
    async fn test_insert_fill() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let fill = Fill {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            order_id: Some(1),
            strategy_id: "test".to_string(),
            price: Decimal::new(10000, 2).into(),
            quantity: Decimal::new(105, 1).into(),
            commission: Decimal::new(10, 2).into(),
        };

        manager.insert_fill(fill).await.unwrap();

        // Check that the fill was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fills")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }
}
