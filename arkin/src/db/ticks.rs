use crate::models::{Instrument, Tick};
use anyhow::Result;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::error;

use super::DBManager;

#[derive(Debug, sqlx::FromRow)]
struct TickRow {
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    tick_id: i64,
    bid_price: Decimal,
    bid_quantity: Decimal,
    ask_price: Decimal,
    ask_quantity: Decimal,
    source: String,
}

impl From<Tick> for TickRow {
    fn from(tick: Tick) -> Self {
        Self {
            event_time: tick.event_time,
            instrument_type: tick.instrument.instrument_type().to_string(),
            venue: tick.instrument.venue().to_string(),
            base: tick.instrument.base().to_string(),
            quote: tick.instrument.quote().to_string(),
            maturity: tick.instrument.maturity().map(|m| m.value()),
            strike: tick.instrument.strike().map(|s| s.value()),
            option_type: tick.instrument.option_type().map(|ot| ot.to_string()),
            tick_id: tick.tick_id as i64,
            bid_price: tick.bid_price.value(),
            bid_quantity: tick.bid_quantity.value(),
            ask_price: tick.ask_price.value(),
            ask_quantity: tick.ask_quantity.value(),
            source: tick.source.to_string(),
        }
    }
}

impl From<TickRow> for Tick {
    fn from(db_tick: TickRow) -> Self {
        let instrument = Instrument::new(
            db_tick.instrument_type.parse().expect("Failed to parse instrument type"),
            db_tick.venue.parse().expect("Falied to parse venue"),
            db_tick.base.as_str().into(),
            db_tick.quote.as_str().into(),
            db_tick.maturity.map(|m| m.into()),
            db_tick.strike.map(|s| s.into()),
            db_tick.option_type.map(|ot| ot.parse().unwrap()),
        )
        .expect("Failed to create instrument");

        Tick {
            event_time: db_tick.event_time,
            instrument,
            tick_id: db_tick.tick_id as u64,
            bid_price: db_tick.bid_price.into(),
            bid_quantity: db_tick.bid_quantity.into(),
            ask_price: db_tick.ask_price.into(),
            ask_quantity: db_tick.ask_quantity.into(),
            source: db_tick.source.parse().expect("Invalid source"),
        }
    }
}

impl DBManager {
    pub async fn insert_tick(&self, tick: Tick) -> Result<()> {
        let tick = TickRow::from(tick);
        sqlx::query!(
            r#"
            WITH existing_instrument AS (
                SELECT instrument_id
                FROM instruments
                WHERE instrument_type = $2
                AND venue = $3
                AND base = $4
                AND quote = $5
                AND maturity IS NOT DISTINCT FROM $6
                AND strike IS NOT DISTINCT FROM $7
                AND option_type IS NOT DISTINCT FROM $8
            ), insert_instrument AS (
                INSERT INTO instruments (instrument_type, venue, base, quote, maturity, strike, option_type)
                SELECT $2, $3, $4, $5, $6, $7, $8
                WHERE NOT EXISTS (SELECT 1 FROM existing_instrument)
                RETURNING instrument_id
            )
            INSERT INTO ticks (
                event_time, instrument_id, tick_id, bid_price, bid_quantity, ask_price, ask_quantity, source
            )
            SELECT 
                $1, COALESCE(ei.instrument_id, ii.instrument_id), $9, $10, $11, $12, $13, $14
            FROM 
                existing_instrument ei
            FULL OUTER JOIN 
                insert_instrument ii ON true
            LIMIT 1
            "#,
            tick.event_time,
            tick.instrument_type,
            tick.venue,
            tick.base,
            tick.quote,
            tick.maturity,
            tick.strike,
            tick.option_type,
            tick.tick_id,
            tick.bid_price,
            tick.bid_quantity,
            tick.ask_price,
            tick.ask_quantity,
            tick.source
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_ticks_batch(&self, ticks: Vec<Tick>) -> Result<()> {
        let ticks = ticks.into_iter().map(TickRow::from).collect::<Vec<_>>();

        let mut tx = self.pool.begin().await?;
        for tick in ticks {
            sqlx::query(
                r#"
                WITH existing_instrument AS (
                    SELECT instrument_id
                    FROM instruments
                    WHERE instrument_type = $2
                    AND venue = $3
                    AND base = $4
                    AND quote = $5
                    AND maturity IS NOT DISTINCT FROM $6
                    AND strike IS NOT DISTINCT FROM $7
                    AND option_type IS NOT DISTINCT FROM $8
                ), insert_instrument AS (
                    INSERT INTO instruments (instrument_type, venue, base, quote, maturity, strike, option_type)
                    SELECT $2, $3, $4, $5, $6, $7, $8
                    WHERE NOT EXISTS (SELECT 1 FROM existing_instrument)
                    RETURNING instrument_id
                )
                INSERT INTO ticks (
                    received_time, event_time, instrument_id, tick_id, bid_price, bid_quantity, ask_price, ask_quantity, source
                )
                SELECT 
                    $1, COALESCE(ei.instrument_id, ii.instrument_id), $9, $10, $11, $12, $13, $14
                FROM 
                    existing_instrument ei
                FULL OUTER JOIN 
                    insert_instrument ii ON true
                LIMIT 1
                "#)
                .bind(tick.event_time)
                .bind(tick.instrument_type)
                .bind(tick.venue)
                .bind(tick.base)
                .bind(tick.quote)
                .bind(tick.maturity)
                .bind(tick.strike)
                .bind(tick.option_type)
                .bind(tick.tick_id)
                .bind(tick.bid_price)
                .bind(tick.bid_quantity)
                .bind(tick.ask_price)
                .bind(tick.ask_quantity)
                .bind(tick.source)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn read_ticks(&self, from: OffsetDateTime, till: OffsetDateTime) -> Vec<Tick> {
        let stream = sqlx::query_as::<_, TickRow>(
            r#"
            SELECT 
                ticks.event_time, 
                instruments.instrument_type, 
                instruments.venue, 
                instruments.base, 
                instruments.quote, 
                instruments.maturity, 
                instruments.strike, 
                instruments.option_type, 
                ticks.tick_id,
                ticks.bid_price, 
                ticks.bid_quantity, 
                ticks.ask_price, 
                ticks.ask_quantity, 
                ticks.source
            FROM ticks
            JOIN instruments ON ticks.instrument_id = instruments.instrument_id
            WHERE ticks.event_time >= $1 AND ticks.event_time < $2
            "#,
        )
        .bind(from)
        .bind(till)
        .fetch(&self.pool);

        stream
            .filter_map(|res| async {
                match res {
                    Ok(db_tick) => Some(db_tick.into()),
                    Err(e) => {
                        error!("Error reading tick: {:?}", e);
                        None
                    }
                }
            })
            .collect()
            .await
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::*;
    use time::OffsetDateTime;
    use tracing::info;

    use super::*;
    use crate::{
        config,
        ingestors::IngestorID,
        logging,
        models::{Instrument, Venue},
    };

    #[tokio::test]
    #[ignore]
    async fn test_insert_tick() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let tick = Tick {
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
            tick_id: 1,
            bid_price: Decimal::new(10000, 2).into(),
            bid_quantity: Decimal::new(105, 1).into(),
            ask_price: Decimal::new(10001, 2).into(),
            ask_quantity: Decimal::new(106, 1).into(),
            source: IngestorID::Test,
        };

        manager.insert_tick(tick).await.unwrap();

        // Check that the tick was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ticks")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    #[ignore]
    async fn test_read_ticks() {
        logging::init_test_tracing();

        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let till = OffsetDateTime::now_utc();
        let from = till - time::Duration::days(1);

        let ticks = manager.read_ticks(from, till).await;
        assert_eq!(ticks.len(), 2);
        for tick in ticks {
            info!("{}", tick);
        }
    }
}
