use crate::{
    config::DatabaseConfig,
    models::{Tick, Trade},
};
use anyhow::Result;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};
use std::time::Duration;
use tracing::info;

pub struct DBManager {
    pub pool: PgPool,
}

impl DBManager {
    pub async fn from_config(config: &DatabaseConfig) -> Self {
        let conn_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&config.password)
            .database(&config.database)
            .ssl_mode(PgSslMode::Prefer);

        let res = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect_with(conn_options)
            .await;

        let pool = match res {
            Ok(pool) => {
                info!("Connected to database");
                pool
            }
            Err(e) => panic!("SQLX failed to connect to database: {}", e),
        };

        Self { pool }
    }

    pub async fn test(&self) {
        // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&self.pool)
            .await
            .expect("SQLX failed to fetch row");

        assert_eq!(row.0, 150);
    }

    pub async fn insert_trade(&self, trade: Trade) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO trades (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, trade_id, price, quantity, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            trade.received_time,
            trade.event_time,
            trade.instrument.instrument_type().to_string(),
            trade.instrument.venue().to_string(),
            trade.instrument.base().to_string(),
            trade.instrument.quote().to_string(),
            trade.instrument.maturity().map(|m| m.value()),
            trade.instrument.strike().map(|s| s.value()),
            trade.instrument.option_type().map(|ot| ot.to_string()),
            trade.trade_id as i64,
            trade.price.value(),
            trade.quantity.value(),
            trade.source.to_string(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_tick(&self, tick: Tick) -> Result<()> {
        sqlx::query!(
            r#"
            insert into ticks (received_time, event_time, instrument_type, venue, base, quote, maturity, strike, option_type, bid_price, bid_quantity, ask_price, ask_quantity, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            tick.received_time,
            tick.event_time,
            tick.instrument.instrument_type().to_string(),
            tick.instrument.venue().to_string(),
            tick.instrument.base().to_string(),
            tick.instrument.quote().to_string(),
            tick.instrument.maturity().map(|m| m.value()),
            tick.instrument.strike().map(|s| s.value()),
            tick.instrument.option_type().map(|ot| ot.to_string()),
            tick.bid_price.value(),
            tick.bid_quantity.value(),
            tick.ask_price.value(),
            tick.ask_quantity.value(),
            tick.source.to_string(),
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
        ingestors::IngestorID,
        models::{Asset, FutureContract, Instrument, Maturity, Price, Quantity, Venue},
    };

    #[tokio::test]
    async fn test_db_manager() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;
        manager.test().await;
    }

    #[tokio::test]
    async fn test_insert_trade() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let trade = Trade {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            trade_id: 1,
            price: Price::new(Decimal::new(10000, 2)).unwrap(),
            quantity: Quantity::new(Decimal::new(105, 1)),
            source: IngestorID::Test,
        };

        manager.insert_trade(trade).await.unwrap();

        // Check that the trade was inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trades")
            .fetch_one(&manager.pool)
            .await
            .expect("SQLX failed to fetch row");
        assert_eq!(row.0, 1)
    }

    #[tokio::test]
    async fn test_insert_tick() {
        let config = config::load();
        let manager = DBManager::from_config(&config.db).await;

        let tick = Tick {
            received_time: OffsetDateTime::now_utc(),
            event_time: OffsetDateTime::now_utc(),
            instrument: Instrument::Future(FutureContract::new(
                &Venue::Binance,
                &Asset::new("BTC"),
                &Asset::new("USDT"),
                &Maturity::new(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )),
            bid_price: Price::new(Decimal::new(10000, 2)).unwrap(),
            bid_quantity: Quantity::new(Decimal::new(105, 1)),
            ask_price: Price::new(Decimal::new(10001, 2)).unwrap(),
            ask_quantity: Quantity::new(Decimal::new(106, 1)),
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
}
