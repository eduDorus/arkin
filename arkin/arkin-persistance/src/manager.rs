use std::{cell::OnceCell, sync::Arc, time::Duration};

use arkin_core::prelude::*;
use futures_util::{stream, StreamExt};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    Error, PgPool,
};
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use crate::{
    config::DatabaseConfig, DBContractType, DBInstrument, DBInstrumentStatus, DBOptionType, DBTick, DBTrade, DBVenue,
};

pub const BIND_LIMIT: usize = 65535;
pub const MAX_CONCURRENT_QUERIES: usize = 10; // Adjust as needed

pub struct PersistanceManager {
    pub pool: PgPool,
    venue_repo: OnceCell<Arc<VenueRepo>>,
    instrument_repo: OnceCell<Arc<InstrumentRepo>>,
    tick_repo: OnceCell<Arc<TickRepo>>,
    trade_repo: OnceCell<Arc<TradeRepository>>,
}

impl PersistanceManager {
    pub fn from_config(config: &DatabaseConfig) -> Self {
        let conn_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&config.password)
            .database(&config.database)
            .ssl_mode(PgSslMode::Prefer);

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect_lazy_with(conn_options);

        Self {
            pool,
            venue_repo: OnceCell::new(),
            instrument_repo: OnceCell::new(),
            tick_repo: OnceCell::new(),
            trade_repo: OnceCell::new(),
        }
    }

    pub fn venue_repo(&self) -> Arc<VenueRepo> {
        self.venue_repo.get_or_init(|| Arc::new(VenueRepo::new())).clone()
    }

    pub fn instrument_repo(&self) -> Arc<InstrumentRepo> {
        self.instrument_repo.get_or_init(|| Arc::new(InstrumentRepo::new())).clone()
    }

    pub fn tick_repo(&self) -> Arc<TickRepo> {
        self.tick_repo.get_or_init(|| Arc::new(TickRepo::new())).clone()
    }

    pub fn trade_repo(&self) -> Arc<TradeRepository> {
        self.trade_repo.get_or_init(|| Arc::new(TradeRepository::new())).clone()
    }
}

pub struct VenueRepo {}

impl VenueRepo {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn insert(&self, pool: &PgPool, venue: Venue) -> Result<(), Error> {
        let venue = DBVenue::from(venue);
        sqlx::query!(
            r#"
            INSERT INTO venues (id, name, venue_type)
            VALUES ($1, $2, $3)
            "#,
            venue.id,
            venue.name,
            venue.venue_type,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<DBVenue>, Error> {
        let venue = sqlx::query_as!(
            DBVenue,
            r#"
            SELECT * FROM venues
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(venue)
    }
}

pub struct InstrumentRepo {}

impl InstrumentRepo {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn insert(&self, pool: &PgPool, instrument: Instrument) -> Result<(), Error> {
        let instrument = DBInstrument::from(instrument);
        sqlx::query!(
            r#"
            INSERT INTO instruments (
                id, venue, symbol, venue_symbol, contract_type, base_asset, quote_asset, strike, maturity, option_type,
                contract_size, price_precision, quantity_precision, base_precision, quote_precision, lot_size, tick_size, status
            ) VALUES (
                $1, $2, $3, $4, $5::contract_type, $6, $7, $8, $9, $10::option_type,
                $11, $12, $13, $14, $15, $16, $17, $18::instrument_status
            )
            "#,
            instrument.id,
            instrument.venue,
            instrument.symbol,
            instrument.venue_symbol,
            instrument.contract_type as DBContractType,
            instrument.base_asset,
            instrument.quote_asset,
            instrument.strike,
            instrument.maturity,
            instrument.option_type as Option<DBOptionType>,
            instrument.contract_size,
            instrument.price_precision,
            instrument.quantity_precision,
            instrument.base_precision,
            instrument.quote_precision,
            instrument.lot_size,
            instrument.tick_size,
            instrument.status as DBInstrumentStatus
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<DBInstrument>, Error> {
        let instrument = sqlx::query_as!(
            DBInstrument,
            r#"
            SELECT
                id,
                venue,
                symbol,
                venue_symbol,
                contract_type AS "contract_type:DBContractType",
                base_asset,
                quote_asset,
                strike,
                maturity,
                option_type AS "option_type:DBOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:DBInstrumentStatus"
            FROM instruments
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(instrument)
    }
}

pub struct TickRepo {}

impl TickRepo {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn insert(&self, pool: &PgPool, tick: Tick) -> Result<(), Error> {
        let tick = DBTick::from(tick);
        sqlx::query!(
            r#"
            INSERT INTO ticks (instrument_id, event_time, tick_id, bid_price, bid_quantity, ask_price, ask_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            tick.instrument_id,
            tick.event_time,
            tick.tick_id,
            tick.bid_price,
            tick.bid_quantity,
            tick.ask_price,
            tick.ask_quantity,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, pool: &PgPool, ticks: Vec<Tick>) -> Result<(), Error> {
        let ticks = ticks.into_iter().map(DBTick::from).collect::<Vec<_>>();

        let queries = ticks
            .chunks(BIND_LIMIT / 7)
            .map(|batch| {
                // Create a query builder
                let mut query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO ticks (instrument_id, event_time, tick_id, bid_price, bid_quantity, ask_price, ask_quantity) ",
                );

                // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
                query_builder.push_values(batch, |mut b, trade| {
                    // If you wanted to bind these by-reference instead of by-value,
                    // you'd need an iterator that yields references that live as long as `query_builder`,
                    // e.g. collect it to a `Vec` first.
                    b.push_bind(trade.instrument_id)
                        .push_bind(trade.event_time)
                        .push_bind(trade.tick_id)
                        .push_bind(trade.bid_price)
                        .push_bind(trade.bid_quantity)
                        .push_bind(trade.ask_price)
                        .push_bind(trade.ask_quantity); 
                });

                query_builder
            })
            .collect::<Vec<_>>();

        let query_stream = stream::iter(queries.into_iter().map(|mut query| {
            let db_pool = pool.clone();
            async move { query.build().execute(&db_pool).await }
        }));

        let results = query_stream.buffer_unordered(MAX_CONCURRENT_QUERIES).collect::<Vec<_>>().await;

        for result in results {
            match result {
                Ok(_) => { /* Success */ }
                Err(e) => {
                    error!("Error executing query: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub async fn read_range(
        &self,
        pool: &PgPool,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DBTick>, Error> {
        let ticks = sqlx::query_as!(
            DBTick,
            r#"
            SELECT * FROM ticks
            WHERE event_time >= $1 AND event_time <= $2
            "#,
            start,
            end,
        )
        .fetch_all(pool)
        .await?;

        Ok(ticks)
    }
}

pub struct TradeRepository {}

impl TradeRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn insert(&self, pool: &PgPool, trade: Trade) -> Result<(), Error> {
        let trade = DBTrade::from(trade);
        sqlx::query!(
            r#"
            INSERT INTO trades (instrument_id,  event_time, trade_id, price, quantity)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            trade.instrument_id,
            trade.event_time,
            trade.trade_id,
            trade.price,
            trade.quantity,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, pool: &PgPool, trades: Vec<Trade>) -> Result<(), Error> {
        let trades = trades.into_iter().map(DBTrade::from).collect::<Vec<_>>();

        let queries = trades
            .chunks(BIND_LIMIT / 5)
            .map(|batch| {
                // Create a query builder
                let mut query_builder = sqlx::QueryBuilder::new(
                    "INSERT INTO trades (instrument_id,  event_time, trade_id, price, quantity) ",
                );

                // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
                query_builder.push_values(batch, |mut b, trade| {
                    // If you wanted to bind these by-reference instead of by-value,
                    // you'd need an iterator that yields references that live as long as `query_builder`,
                    // e.g. collect it to a `Vec` first.
                    b.push_bind(trade.instrument_id)
                        .push_bind(trade.event_time)
                        .push_bind(trade.trade_id)
                        .push_bind(trade.price)
                        .push_bind(trade.quantity);
                });

                query_builder
            })
            .collect::<Vec<_>>();

        let query_stream = stream::iter(queries.into_iter().map(|mut query| {
            let db_pool = pool.clone();
            async move { query.build().execute(&db_pool).await }
        }));

        let results = query_stream.buffer_unordered(MAX_CONCURRENT_QUERIES).collect::<Vec<_>>().await;

        for result in results {
            match result {
                Ok(_) => { /* Success */ }
                Err(e) => {
                    error!("Error executing query: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub async fn list_range(
        &self,
        pool: &PgPool,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DBTrade>, Error> {
        let trades = sqlx::query_as!(
            DBTrade,
            r#"
            SELECT * FROM trades
            WHERE event_time >= $1 AND event_time <= $2
            "#,
            start,
            end,
        )
        .fetch_all(pool)
        .await?;

        Ok(trades)
    }
}
