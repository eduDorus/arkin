use super::DBManager;
use crate::models::Allocation;
use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
struct AllocationRow {
    received_time: OffsetDateTime,
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    strategy_id: String,
    notional: Decimal,
}

impl From<Allocation> for AllocationRow {
    fn from(allocation: Allocation) -> Self {
        Self {
            received_time: allocation.received_time,
            event_time: allocation.event_time,
            instrument_type: allocation.instrument.instrument_type().to_string(),
            venue: allocation.instrument.venue().to_string(),
            base: allocation.instrument.base().to_string(),
            quote: allocation.instrument.quote().to_string(),
            maturity: allocation.instrument.maturity().map(|m| m.value()),
            strike: allocation.instrument.strike().map(|s| s.value()),
            option_type: allocation.instrument.option_type().map(|ot| ot.to_string()),
            strategy_id: allocation.strategy_id,
            notional: allocation.notional.value(),
        }
    }
}

impl DBManager {
    pub async fn insert_allocation(&self, allocation: Allocation) -> Result<()> {
        let allocation = AllocationRow::from(allocation);
        sqlx::query!(
            r#"
            INSERT INTO allocations (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, strategy_id, notional)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            allocation.received_time,
            allocation.event_time,
            allocation.instrument_type,
            allocation.venue,
            allocation.base,
            allocation.quote,
            allocation.maturity,
            allocation.strike,
            allocation.option_type,
            allocation.strategy_id,
            allocation.notional,
        ).execute(&self.pool).await?;

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
    async fn test_insert_allocation() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let allocation = Allocation {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            strategy_id: "test".to_string(),
            notional: Decimal::new(100000, 2).into(),
        };

        manager.insert_allocation(allocation).await.unwrap();

        // Check that the allocation was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM allocations")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }
}
