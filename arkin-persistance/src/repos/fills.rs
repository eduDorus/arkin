use super::DBManager;
use crate::models::Fill;
use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
struct FillRow {
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    order_id: i64,
    strategy_id: String,
    price: Decimal,
    quantity: Decimal,
    commission: Decimal,
}

impl From<Fill> for FillRow {
    fn from(fill: Fill) -> Self {
        Self {
            event_time: fill.event_time,
            instrument_type: fill.instrument.instrument_type().to_string(),
            venue: fill.instrument.venue().to_string(),
            base: fill.instrument.base().to_string(),
            quote: fill.instrument.quote().to_string(),
            maturity: fill.instrument.maturity().map(|m| m.value()),
            strike: fill.instrument.strike().map(|s| s.value()),
            option_type: fill.instrument.option_type().map(|ot| ot.to_string()),
            order_id: fill.order_id as i64,
            strategy_id: fill.strategy_id.to_string(),
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
            INSERT INTO fills (event_time, instrument_type, venue, base, quote, maturity, strike, option_type, order_id, strategy_id, price, quantity, commission)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
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
    #[ignore]
    async fn test_insert_fill() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let fill = Fill {
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            order_id: 1,
            strategy_id: "test".into(),
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
