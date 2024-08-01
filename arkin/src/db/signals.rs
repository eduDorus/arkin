use super::DBManager;
use crate::models::Signal;
use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
struct SignalRow {
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
    signal: Decimal,
}

impl From<Signal> for SignalRow {
    fn from(signal: Signal) -> Self {
        Self {
            received_time: signal.received_time,
            event_time: signal.event_time,
            instrument_type: signal.instrument.instrument_type().to_string(),
            venue: signal.instrument.venue().to_string(),
            base: signal.instrument.base().to_string(),
            quote: signal.instrument.quote().to_string(),
            maturity: signal.instrument.maturity().map(|m| m.value()),
            strike: signal.instrument.strike().map(|s| s.value()),
            option_type: signal.instrument.option_type().map(|ot| ot.to_string()),
            strategy_id: signal.strategy_id,
            signal: signal.signal.value(),
        }
    }
}
impl DBManager {
    pub async fn insert_signal(&self, signal: Signal) -> Result<()> {
        let signal = SignalRow::from(signal);
        sqlx::query!(
            r#"
            INSERT INTO signals (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, strategy_id, signal)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            signal.received_time,
            signal.event_time,
            signal.instrument_type,
            signal.venue,
            signal.base,
            signal.quote,
            signal.maturity,
            signal.strike,
            signal.option_type,
            signal.strategy_id,
            signal.signal,
        )
        .execute(&self.pool)
        .await?;

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
    async fn test_insert_signal() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let signal = Signal {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            strategy_id: "test".to_string(),
            signal: Decimal::new(1, 0).into(),
        };

        manager.insert_signal(signal).await.unwrap();

        // Check that the signal was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM signals")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }
}
