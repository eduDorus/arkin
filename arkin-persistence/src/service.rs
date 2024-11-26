use std::{sync::Arc, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::repos::{InsightsRepo, InstrumentRepo, TickRepo, TradeRepo, VenueRepo};
use crate::services::TradeService;
use crate::services::{InsightsService, InstrumentService, TickService, VenueService};
use crate::traits::Persistor;
use crate::{PersistenceConfig, PersistenceError};

#[derive(Debug)]
pub struct PersistenceService {
    pub pubsub: Arc<PubSub>,
    pub auto_commit_interval: Duration,
    // venue_service: Arc<VenueService>,
    pub instrument_service: Arc<InstrumentService>,
    pub tick_service: Arc<TickService>,
    pub trade_service: Arc<TradeService>,
    pub insights_service: Arc<InsightsService>,
}

impl PersistenceService {
    pub fn from_config(config: &PersistenceConfig, pubsub: Arc<PubSub>) -> Self {
        let db_config = config.database.clone();
        let conn_options = PgConnectOptions::new()
            .host(&db_config.host)
            .port(db_config.port)
            .username(&db_config.user)
            .password(&db_config.password)
            .database(&db_config.database)
            .ssl_mode(PgSslMode::Prefer);

        let pool = PgPoolOptions::new()
            .min_connections(db_config.min_connections)
            .max_connections(db_config.max_connections)
            .idle_timeout(Duration::from_secs(db_config.idle_timeout))
            .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
            .max_lifetime(Duration::from_secs(db_config.max_lifetime))
            .connect_lazy_with(conn_options);

        // Initialize repositories
        let venue_repo = VenueRepo::new(pool.clone());
        let instrument_repo = InstrumentRepo::new(pool.clone());
        let tick_repo = TickRepo::new(pool.clone());
        let trade_repo = Arc::new(TradeRepo::new(pool.clone()));
        let insights_repo = Arc::new(InsightsRepo::new(pool.clone()));

        // Initialize services
        let venue_service = VenueService::new(venue_repo);
        let instrument_service = Arc::new(InstrumentService::new(instrument_repo, venue_service));
        let tick_service = Arc::new(TickService::new(tick_repo, instrument_service.clone(), config.batch_size));
        let trade_service = Arc::new(TradeService::new(
            trade_repo.clone(),
            instrument_service.clone(),
            config.batch_size,
        ));
        let insights_service = Arc::new(InsightsService::new(insights_repo.clone(), config.batch_size));

        Self {
            pubsub,
            auto_commit_interval: Duration::from_secs(config.auto_commit_interval),
            // venue_service,
            instrument_service,
            tick_service,
            trade_service,
            insights_service,
        }
    }
}

#[async_trait]
impl Persistor for PersistenceService {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PersistenceError> {
        info!("Starting persistence service...");
        let tick_service = self.tick_service.clone();
        let trade_service = self.trade_service.clone();
        let insights_service = self.insights_service.clone();

        let mut interval = tokio::time::interval(self.auto_commit_interval);

        let mut trades = self.pubsub.subscribe::<Trade>();
        let mut ticks = self.pubsub.subscribe::<Tick>();
        let mut insights = self.pubsub.subscribe::<Insight>();
        let mut insights_tick = self.pubsub.subscribe::<InsightTick>();

        loop {
            tokio::select! {
                    Ok(trade) = trades.recv() => {
                        if let Err(e) = trade_service.insert_batch(trade).await {
                            error!("Failed to insert trade: {}", e);
                        }
                    }
                    Ok(tick) = ticks.recv() => {
                        if let Err(e) = tick_service.insert_batch(tick).await {
                            error!("Failed to insert tick: {}", e);
                        }
                    }
                    Ok(insight) = insights.recv() => {
                        if let Err(e) = insights_service.insert_batch(insight).await {
                            error!("Failed to insert insight: {}", e);
                        }
                    }
                    Ok(insight_tick) = insights_tick.recv() => {
                        if let Err(e) = insights_service.insert_batch_vec(insight_tick.insights).await {
                            error!("Failed to insert insight tick: {}", e);
                        }
                    }
                    _ = interval.tick() => {
                        debug!("Auto commit persistence service...");
                        if let Err(e) = self.flush().await {
                            error!("Failed to auto commit persistence service: {}", e);
                        }
                    }
                    _ = shutdown.cancelled() => {
                        if let Err(e) = self.flush().await {
                            error!("Failed to commit persistence service on shutdown: {}", e);
                        }
                        break;
                    }
            }
        }

        info!("Persistence service started");
        Ok(())
    }

    async fn flush(&self) -> Result<(), PersistenceError> {
        self.tick_service.flush().await?;
        self.trade_service.flush().await?;
        self.insights_service.flush().await?;
        Ok(())
    }

    async fn insert_instrument(&self, instrument: Instrument) -> Result<(), PersistenceError> {
        self.instrument_service.insert(instrument).await.map_err(|e| e.into())
    }

    async fn read_instrument_by_id(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        self.instrument_service.read_by_id(id).await.map_err(|e| e.into())
    }

    async fn read_instrument_by_venue_symbol(&self, venue_symbol: String) -> Result<Arc<Instrument>, PersistenceError> {
        let instrument_service = &self.instrument_service;

        instrument_service
            .read_by_venue_symbol(venue_symbol)
            .await
            .map_err(|e| e.into())
    }

    async fn insert_tick(&self, tick: Tick) -> Result<(), PersistenceError> {
        self.tick_service.insert(tick).await.map_err(|e| e.into())
    }

    async fn insert_tick_batch(&self, tick: Tick) -> Result<(), PersistenceError> {
        let tick_service = &self.tick_service;
        tick_service.insert_batch(tick).await.map_err(|e| e.into())
    }

    async fn insert_tick_batch_vec(&self, ticks: Vec<Tick>) -> Result<(), PersistenceError> {
        self.tick_service.insert_batch_vec(ticks).await.map_err(|e| e.into())
    }

    async fn read_latest_tick(
        &self,
        event_time: OffsetDateTime,
        instrument: &Arc<Instrument>,
    ) -> Result<Option<Tick>, PersistenceError> {
        self.tick_service
            .read_latest_tick(event_time, instrument)
            .await
            .map_err(|e| e.into())
    }

    async fn read_trades_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Trade>, PersistenceError> {
        self.trade_service.read_range(instruments, from, to).await.map_err(|e| e.into())
    }

    async fn insert_trade(&self, trade: Trade) -> Result<(), PersistenceError> {
        self.trade_service.insert(trade).await.map_err(|e| e.into())
    }

    async fn insert_trade_batch(&self, trade: Trade) -> Result<(), PersistenceError> {
        let trade_service = &self.trade_service;
        trade_service.insert_batch(trade).await.map_err(|e| e.into())
    }

    async fn insert_trade_batch_vec(&self, trades: Vec<Trade>) -> Result<(), PersistenceError> {
        self.trade_service.insert_batch_vec(trades).await.map_err(|e| e.into())
    }

    async fn read_ticks_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Tick>, PersistenceError> {
        self.tick_service.read_range(instruments, from, to).await.map_err(|e| e.into())
    }

    async fn insert_insight(&self, insight: Insight) -> Result<(), PersistenceError> {
        self.insights_service.insert(insight).await.map_err(|e| e.into())
    }

    async fn insert_insight_batch(&self, insight: Insight) -> Result<(), PersistenceError> {
        self.insights_service.insert_batch(insight).await.map_err(|e| e.into())
    }

    async fn insert_insight_batch_vec(&self, insights: Vec<Insight>) -> Result<(), PersistenceError> {
        self.insights_service.insert_batch_vec(insights).await.map_err(|e| e.into())
    }
}
