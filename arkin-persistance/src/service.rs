use std::{sync::Arc, time::Duration};

use anyhow::Result;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::repos::{InsightsRepo, InstrumentRepo, TickRepo, TradeRepo, VenueRepo};
use crate::services::{InsightsService, InstrumentService, TickService, VenueService};
use crate::{config::DatabaseConfig, services::TradeService};

pub struct PersistanceService {
    venue_service: Arc<VenueService>,
    instrument_service: Arc<InstrumentService>,
    tick_service: Arc<TickService>,
    trade_service: Arc<TradeService>,
    insights_service: Arc<InsightsService>,
}

impl PersistanceService {
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

        // Initialize repositories
        let venue_repo = Arc::new(VenueRepo::new(pool.clone()));
        let instrument_repo = Arc::new(InstrumentRepo::new(pool.clone()));
        let tick_repo = Arc::new(TickRepo::new(pool.clone()));
        let trade_repo = Arc::new(TradeRepo::new(pool.clone()));
        let insights_repo = Arc::new(InsightsRepo::new(pool.clone()));

        // Initialize services
        let venue_service = Arc::new(VenueService::new(venue_repo.clone()));
        let instrument_service = Arc::new(InstrumentService::new(instrument_repo.clone(), venue_service.clone()));
        let tick_service = Arc::new(TickService::new(tick_repo.clone(), instrument_service.clone()));
        let trade_service = Arc::new(TradeService::new(trade_repo.clone(), instrument_service.clone()));
        let insights_service = Arc::new(InsightsService::new(insights_repo.clone(), instrument_service.clone()));

        Self {
            venue_service,
            instrument_service,
            tick_service,
            trade_service,
            insights_service,
        }
    }

    pub async fn insert_venue(&self, venue: Venue) -> Result<()> {
        self.venue_service.insert(venue).await
    }

    pub async fn read_venue_by_id(&self, id: &Uuid) -> Result<Option<Venue>> {
        self.venue_service.read_by_id(id).await
    }

    pub async fn insert_instrument(&self, instrument: Instrument) -> Result<()> {
        self.instrument_service.insert(instrument).await
    }

    pub async fn read_instrument_by_id(&self, id: &Uuid) -> Result<Option<Instrument>> {
        self.instrument_service.read_by_id(id).await
    }

    pub async fn read_instrument_by_venue_symbol(&self, venue_symbol: &str) -> Result<Option<Instrument>> {
        debug!(
            "PersistanceService asking instrument service for venue symbol: {}",
            venue_symbol
        );
        self.instrument_service.read_by_venue_symbol(venue_symbol).await
    }

    pub async fn insert_tick(&self, tick: Tick) -> Result<()> {
        self.tick_service.insert(tick).await
    }

    pub async fn insert_tick_batch(&self, ticks: Vec<Tick>) -> Result<()> {
        self.tick_service.insert_batch(ticks).await
    }

    pub async fn read_trades_range(
        &self,
        instrument_ids: &[Uuid],
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Trade>> {
        self.trade_service.read_range(instrument_ids, from, to).await
    }

    pub async fn insert_trade(&self, trade: Trade) -> Result<()> {
        self.trade_service.insert(trade).await
    }

    pub async fn insert_trade_batch(&self, trades: Vec<Trade>) -> Result<()> {
        self.trade_service.insert_batch(trades).await
    }

    pub async fn read_ticks_range(
        &self,
        instrument_ids: &[Uuid],
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Tick>> {
        self.tick_service.read_range(instrument_ids, from, to).await
    }

    pub async fn insert_insight(&self, insight: Insight) -> Result<()> {
        self.insights_service.insert(insight).await
    }

    pub async fn insert_insight_batch(&self, insights: Vec<Insight>) -> Result<()> {
        self.insights_service.insert_batch(insights).await
    }

    pub async fn read_insights_range_by_instrument_id_and_feature_id(
        &self,
        instrument_id: &Uuid,
        feature_id: &str,
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Insight>> {
        self.insights_service
            .read_range_by_instrument_id_and_feature_id(instrument_id, feature_id, from, to)
            .await
    }
}
