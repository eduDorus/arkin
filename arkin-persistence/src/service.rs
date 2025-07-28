use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use clickhouse::Client;
use futures::Stream;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::ConnectOptions;
use time::UtcDateTime;
use tracing::{debug, error, info, instrument, warn};

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::context::PersistenceContext;
use crate::stores::*;
use crate::{PersistenceConfig, PersistenceError};

pub struct Persistence {
    identifier: String,
    mode: InstanceType,
    dry_run: bool,
    only_normalized: bool,
    only_predictions: bool,
    flush_interval: Duration,
    ctx: PersistenceContext,
}

impl Persistence {
    pub fn new(
        config: &PersistenceConfig,
        instance: Instance,
        only_normalized: bool,
        only_predictions: bool,
        dry_run: bool,
    ) -> Arc<Self> {
        let pg_config = config.postgres.clone();
        let conn_options = PgConnectOptions::new()
            .host(&pg_config.host)
            .port(pg_config.port)
            .username(&pg_config.user)
            .password(&pg_config.password)
            .database(&pg_config.database)
            .ssl_mode(PgSslMode::Prefer)
            .log_statements("DEBUG".parse().unwrap())
            .log_slow_statements("DEBUG".parse().unwrap(), Duration::from_secs(300));

        let pg_pool = PgPoolOptions::new()
            .min_connections(pg_config.min_connections)
            .max_connections(pg_config.max_connections)
            .idle_timeout(Duration::from_secs(pg_config.idle_timeout))
            .acquire_timeout(Duration::from_secs(pg_config.acquire_timeout))
            .max_lifetime(Duration::from_secs(pg_config.max_lifetime))
            .connect_lazy_with(conn_options);

        let ch_config = config.clickhouse.clone();
        let ch_client = Client::default()
            .with_url(format!("http://{}:{}", ch_config.host, ch_config.port))
            .with_compression(clickhouse::Compression::Lz4)
            .with_database(ch_config.database)
            .with_user(ch_config.user)
            .with_password(ch_config.password)
            .with_option("wait_end_of_query", "1");

        let ctx = PersistenceContext::new(pg_pool.clone(), ch_client.clone(), instance.clone().into());

        Self {
            identifier: "persistence".to_owned(),
            mode: instance.instance_type,
            dry_run,
            only_normalized,
            only_predictions,
            flush_interval: Duration::from_secs(ch_config.flush_interval),
            ctx,
        }
        .into()
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_instance_by_name(&self, name: &str) -> Result<Arc<Instance>, PersistenceError> {
        instance_store::read_by_name(&self.ctx, name).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_pipeline(&self, id: Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_id(&self.ctx, &id).await // Assume impl added
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_strategy(&self, id: Uuid) -> Result<Arc<Strategy>, PersistenceError> {
        strategy_store::read_by_id(&self.ctx, &id).await // Assume impl added
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_instrument(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_id(&self.ctx, &id).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_instrument_by_venue_symbol(&self, symbol: &str) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_venue_symbol(&self.ctx, &symbol).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_venue(&self, id: Uuid) -> Result<Arc<Venue>, PersistenceError> {
        venue_store::read_by_id(&self.ctx, &id).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn get_asset(&self, id: Uuid) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::read_by_id(&self.ctx, &id).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn tick_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> impl Stream<Item = Arc<Tick>> + 'static {
        tick_store::stream_range_buffered(&self.ctx, instruments, start, end, buffer_size, frequency).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn trade_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> impl Stream<Item = Arc<AggTrade>> + 'static {
        trade_store::stream_range_buffered(&self.ctx, instruments, start, end, buffer_size, frequency).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_account(&self, account: Arc<Account>) {
        if let Err(e) = account_store::insert(&self.ctx, account).await {
            error!(target: "persistence", "error in inserting account: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_transfer(&self, transfer: Arc<Transfer>) {
        if let Err(e) = transfer_store::insert(&self.ctx, transfer).await {
            error!(target: "persistence", "error in inserting transfer group: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_transfer_batch(&self, batch: Arc<TransferBatch>) {
        if let Err(e) = transfer_store::insert_batch(&self.ctx, batch.transfers.clone()).await {
            error!(target: "persistence", "error in inserting transfer group: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_execution_order(&self, order: Arc<ExecutionOrder>) {
        if let Err(e) = execution_order_store::insert(&self.ctx, order).await {
            error!(target: "persistence", "error in inserting execution order: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn update_execution_order(&self, order: Arc<ExecutionOrder>) {
        if let Err(e) = execution_order_store::update(&self.ctx, order).await {
            error!(target: "persistence", "error in update execution order: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_venue_order(&self, order: Arc<VenueOrder>) {
        if let Err(e) = venue_order_store::insert(&self.ctx, order).await {
            error!(target: "persistence", "error in inserting venue order: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn update_venue_order(&self, order: Arc<VenueOrder>) {
        if let Err(e) = venue_order_store::update(&self.ctx, order).await {
            error!(target: "persistence", "error in update venue order: {}",e);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_tick(&self, tick: Arc<Tick>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.ticks.lock().await;
            lock.push(tick);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_trade(&self, trade: Arc<AggTrade>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.trades.lock().await;
            lock.push(trade);
            drop(lock);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn insert_insights_update(&self, tick: Arc<InsightsUpdate>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let insights = if self.only_normalized {
                tick.insights
                    .iter()
                    .filter(|i| i.insight_type == InsightType::Normalized)
                    .cloned()
                    .collect::<Vec<_>>()
            } else if self.only_predictions {
                tick.insights
                    .iter()
                    .filter(|i| i.insight_type == InsightType::Prediction)
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                tick.insights.iter().cloned().collect::<Vec<_>>()
            };
            let mut lock = self.ctx.buffer.insights.lock().await;
            lock.extend(insights);
        }
    }

    // TODO: WE NEED TO FLUSH ALSO TRADES AND TICKS
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn flush_all(&self, ctx: Arc<ServiceCtx>) {
        info!(target: "persistence", "flushing...");
        let insights = {
            let mut lock = self.ctx.buffer.insights.lock().await;
            let insights = std::mem::take(&mut *lock);
            debug!(target: "persistence", "insights buffer length {}", lock.len());
            insights
        };

        if !insights.is_empty() {
            let persistence_ctx = self.ctx.clone();
            ctx.spawn(async move {
                debug!(target: "persistence", "flushing {} insights", insights.len());

                // Insert the insights into the database
                loop {
                    match insight_store::insert_vec(&persistence_ctx, &insights).await {
                        Ok(_) => {
                            info!(target: "persistence", "successfully flushed {} insights", insights.len());
                            break;
                        }
                        Err(e) => {
                            error!(target: "persistence", "failed to flush insights: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }

        let trades = {
            let mut lock = self.ctx.buffer.trades.lock().await;
            let trades = std::mem::take(&mut *lock);
            info!(target: "persistence", "trade buffer length {}", lock.len());
            trades
        };

        if !trades.is_empty() {
            let persistence_ctx = self.ctx.clone();
            ctx.spawn(async move {
                debug!(target: "persistence", "flushing {} trades", trades.len());

                // Insert the insights into the database
                loop {
                    match trade_store::insert_vec(&persistence_ctx, &trades).await {
                        Ok(_) => {
                            info!(target: "persistence", "successfully flushed {} trades", trades.len());
                            break;
                        }
                        Err(e) => {
                            error!(target: "persistence", "failed to flush insights: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }

        let ticks = {
            let mut lock = self.ctx.buffer.ticks.lock().await;
            let ticks = std::mem::take(&mut *lock);
            info!(target: "persistence", "tick buffer length {}", lock.len());
            ticks
        };

        if !ticks.is_empty() {
            let persistence_ctx = self.ctx.clone();
            ctx.spawn(async move {
                debug!(target: "persistence", "flushing {} ticks", ticks.len());

                // Insert the ticks into the database
                loop {
                    match tick_store::insert_vec(&persistence_ctx, &ticks).await {
                        Ok(_) => {
                            info!(target: "persistence", "successfully flushed {} ticks", ticks.len());
                            break;
                        }
                        Err(e) => {
                            error!(target: "persistence", "failed to flush ticks: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    }
}

#[async_trait]
impl Runnable for Persistence {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        if self.dry_run {
            return;
        }

        match event {
            Event::TickUpdate(t) => self.insert_tick(t).await,
            Event::AggTradeUpdate(t) => self.insert_trade(t).await,
            Event::InsightsUpdate(i) => self.insert_insights_update(i).await,

            // Ledger
            Event::NewAccount(a) => self.insert_account(a).await,
            Event::NewTransfer(t) => self.insert_transfer(t).await,
            Event::NewTransferBatch(tb) => self.insert_transfer_batch(tb).await,

            // Execution Orders
            Event::ExecutionOrderBookNew(o) => self.insert_execution_order(o).await,
            Event::ExecutionOrderBookUpdate(o) => self.update_execution_order(o).await,

            // Venue Orders
            Event::VenueOrderBookNew(o) => self.insert_venue_order(o).await,
            Event::VenueOrderBookUpdate(o) => self.update_venue_order(o).await,
            e => warn!(target: "persistence", "received unused event {}", e.event_type()),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn setup(&self, _ctx: Arc<ServiceCtx>) {
        // TODO: NOT FOR PRODUCTION
        if self.ctx.instance.instance_type == InstanceType::Test {
            if let Err(e) = instance_store::delete(&self.ctx, self.ctx.instance.id).await {
                error!(target: "persistence", "could not delete instance: {}", e)
            }
        }
        // Create the instance
        if let Err(e) = instance_store::insert(&self.ctx, self.ctx.instance.clone()).await {
            error!(target: "persistence", "could not create instance: {}", e)
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, ctx: Arc<ServiceCtx>) {
        // Here we spawn the flush task that flushes every x interval (100ms for example)
        let service = self.clone();
        let ctx_clone = ctx.clone();
        ctx.spawn(async move {
            while ctx_clone.is_running().await {
                service.flush_all(ctx_clone.clone()).await;
                tokio::time::sleep(self.flush_interval).await;
            }
        })
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn teardown(&self, ctx: Arc<ServiceCtx>) {
        info!(target: "persistence", "service teardown...");
        info!(target: "persistence", "flushing persistence service on teardown...");
        self.flush_all(ctx).await;
        self.ctx.pg_pool.close().await;
    }
}
