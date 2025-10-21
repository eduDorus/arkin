use std::pin::Pin;
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use clickhouse::Client;
use futures::Stream;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::ConnectOptions;
use time::UtcDateTime;
use tokio::select;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::context::PersistenceContext;
use crate::stores::*;
use crate::PersistenceConfig;

pub struct Persistence {
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
            .log_slow_statements("WARN".parse().unwrap(), Duration::from_secs(300));

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
            .with_option("buffer_size", "1048576")
            .with_validation(false)
            .with_option("wait_end_of_query", "0");

        let ctx = PersistenceContext::new(pg_pool.clone(), ch_client.clone(), instance.clone().into());

        Self {
            mode: instance.instance_type,
            dry_run,
            only_normalized,
            only_predictions,
            flush_interval: Duration::from_secs(ch_config.flush_interval),
            ctx,
        }
        .into()
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn get_pipeline_by_name(&self, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_name(&self.ctx, name).await
    }

    pub async fn insert_pipeline(&self, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
        pipeline_store::insert(&self.ctx, pipeline).await
    }

    pub async fn get_strategy(&self, id: Uuid) -> Result<Arc<Strategy>, PersistenceError> {
        strategy_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn get_instrument(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn list_instruments_by_venue_symbol(
        &self,
        symbols: &[String],
        venue: &Arc<Venue>,
    ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        let mut instruments = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            let inst = instrument_store::read_by_venue_symbol(&self.ctx, symbol, venue).await?;
            instruments.push(inst);
        }
        Ok(instruments)
    }

    pub async fn get_asset(&self, id: Uuid) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn get_scaler_data(
        &self,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        from: UtcDateTime,
        till: UtcDateTime,
        levels: &[f64],
    ) -> Result<Vec<QuantileData>, PersistenceError> {
        scaler_store::get_iqr(&self.ctx, pipeline, instrument, from, till, levels).await
    }

    pub async fn insert_account(&self, account: Arc<Account>) {
        if let Err(e) = account_store::insert(&self.ctx, account).await {
            error!(target: "persistence", "error in inserting account: {}",e);
        }
    }

    pub async fn insert_transfer(&self, transfer: Arc<Transfer>) {
        if let Err(e) = transfer_store::insert(&self.ctx, transfer).await {
            error!(target: "persistence", "error in inserting transfer group: {}",e);
        }
    }

    pub async fn insert_transfer_batch(&self, batch: Arc<TransferBatch>) {
        if let Err(e) = transfer_store::insert_batch(&self.ctx, batch.transfers.clone()).await {
            error!(target: "persistence", "error in inserting transfer group: {}",e);
        }
    }

    pub async fn insert_execution_order(&self, order: Arc<ExecutionOrder>) {
        if let Err(e) = execution_order_store::insert(&self.ctx, order).await {
            error!(target: "persistence", "error in inserting execution order: {}",e);
        }
    }

    pub async fn update_execution_order(&self, order: Arc<ExecutionOrder>) {
        if let Err(e) = execution_order_store::update(&self.ctx, order).await {
            error!(target: "persistence", "error in update execution order: {}",e);
        }
    }

    pub async fn insert_venue_order(&self, order: Arc<VenueOrder>) {
        if let Err(e) = venue_order_store::insert(&self.ctx, order).await {
            error!(target: "persistence", "error in inserting venue order: {}",e);
        }
    }

    pub async fn update_venue_order(&self, order: Arc<VenueOrder>) {
        if let Err(e) = venue_order_store::update(&self.ctx, order).await {
            error!(target: "persistence", "error in update venue order: {}",e);
        }
    }

    pub async fn insert_tick(&self, tick: Arc<Tick>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.ticks.lock().await;
            lock.push(tick);
        }
    }

    pub async fn insert_agg_trade(&self, trade: Arc<AggTrade>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.trades.lock().await;
            lock.push(trade);
            drop(lock);
        }
    }

    pub async fn insert_metric(&self, metric: Arc<Metric>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.metrics.lock().await;
            lock.push(metric);
        }
    }

    pub async fn insert_insights_update(&self, tick: Arc<InsightsUpdate>) {
        if self.mode != InstanceType::Test {
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
                tick.insights.to_vec()
            };
            let mut lock = self.ctx.buffer.insights.lock().await;
            lock.extend(insights);
        }
    }

    // TODO: WE NEED TO FLUSH ALSO TRADES AND TICKS
    pub async fn flush_all(&self, ctx: Arc<ServiceCtx>) {
        debug!(target: "persistence", "flushing...");
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
            debug!(target: "persistence", "trade buffer length {}", lock.len());
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
            debug!(target: "persistence", "tick buffer length {}", lock.len());
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

        let metrics = {
            let mut lock = self.ctx.buffer.metrics.lock().await;
            let metrics = std::mem::take(&mut *lock);
            debug!(target: "persistence", "metric buffer length {}", lock.len());
            metrics
        };

        if !metrics.is_empty() {
            let persistence_ctx = self.ctx.clone();
            ctx.spawn(async move {
                debug!(target: "persistence", "flushing {} metrics", metrics.len());

                // Insert the metrics into the database
                loop {
                    match metric_store::batch_insert_metric(&persistence_ctx, &metrics).await {
                        Ok(_) => {
                            info!(target: "persistence", "successfully flushed {} metrics", metrics.len());
                            break;
                        }
                        Err(e) => {
                            error!(target: "persistence", "failed to flush metrics: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    }
}

async fn flush_task(persistence: Arc<Persistence>, service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
    info!(target: "pubsub", "starting event processor task");
    let token = service_ctx.get_shutdown_token();
    loop {
        persistence.flush_all(service_ctx.clone()).await;
        select! {
            _ = token.cancelled() => break,
            _ = tokio::time::sleep(persistence.flush_interval) => {},
        }
    }
}

#[async_trait]
impl PersistenceReader for Persistence {
    async fn get_instance_by_id(&self, id: &Uuid) -> Result<Arc<Instance>, PersistenceError> {
        instance_store::read_by_id(&self.ctx, id).await
    }

    async fn get_instance_by_name(&self, name: &str) -> Result<Arc<Instance>, PersistenceError> {
        instance_store::read_by_name(&self.ctx, name).await
    }

    async fn get_feature_id(&self, id: &str) -> FeatureId {
        feature_store::read_feature_id(&self.ctx, id).await
    }

    async fn get_pipeline_by_id(&self, id: &Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_id(&self.ctx, id).await
    }

    async fn get_pipeline_by_name(&self, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_name(&self.ctx, name).await
    }

    async fn get_venue_by_id(&self, id: &Uuid) -> Result<Arc<Venue>, PersistenceError> {
        venue_store::read_by_id(&self.ctx, id).await
    }

    async fn get_venue_by_name(&self, name: &VenueName) -> Result<Arc<Venue>, PersistenceError> {
        venue_store::read_by_name(&self.ctx, name).await
    }

    async fn get_instrument_by_id(&self, id: &Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_id(&self.ctx, id).await
    }

    async fn get_instrument_by_venue_symbol(
        &self,
        symbol: &str,
        venue: &Arc<Venue>,
    ) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_venue_symbol(&self.ctx, symbol, venue).await
    }

    async fn get_instruments_by_venue(&self, venue: &Arc<Venue>) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        instrument_store::list_by_venue(&self.ctx, venue).await
    }

    async fn get_instruments_by_venue_and_type(
        &self,
        venue: &Arc<Venue>,
        instrument_type: InstrumentType,
    ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        instrument_store::list_by_venue_and_type(&self.ctx, venue, instrument_type).await
    }

    async fn get_asset_by_id(&self, id: &Uuid) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::read_by_id(&self.ctx, id).await
    }

    async fn get_asset_by_symbol(&self, symbol: &str) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::read_by_symbol(&self.ctx, symbol).await
    }

    async fn get_last_tick(&self, instrument: &Arc<Instrument>) -> Result<Option<Arc<Tick>>, PersistenceError> {
        tick_store::read_last(&self.ctx, instrument).await
    }

    async fn list_trades(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
    ) -> Result<Vec<Arc<AggTrade>>, PersistenceError> {
        trade_store::read_range(&self.ctx, instruments, start, end).await
    }

    async fn tick_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        tick_store::stream_range_buffered(&self.ctx, instruments, start, end, buffer_size, frequency).await
    }

    async fn agg_trade_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        trade_store::stream_range_buffered(&self.ctx, instruments, start, end, buffer_size, frequency).await
    }

    async fn metric_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        metric_type: MetricType,
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        metric_store::stream_range_buffered(&self.ctx, instruments, metric_type, start, end, buffer_size, frequency)
            .await
    }
}

#[async_trait]
impl Runnable for Persistence {
    async fn handle_event(&self, _ctx: Arc<CoreCtx>, event: Event) {
        if self.dry_run {
            return;
        }

        match event {
            Event::TickUpdate(t) => self.insert_tick(t).await,
            Event::AggTradeUpdate(t) => self.insert_agg_trade(t).await,
            Event::InsightsUpdate(i) => self.insert_insights_update(i).await,
            Event::MetricUpdate(i) => self.insert_metric(i).await,

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

    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // TODO: NOT FOR PRODUCTION
        if self.ctx.instance.instance_type == InstanceType::Test
            && let Err(e) = instance_store::delete_by_id(&self.ctx, self.ctx.instance.id).await {
                warn!(target: "persistence", "could not delete instance: {}", e)
            }

        if self.ctx.instance.instance_type == InstanceType::Simulation
            && let Err(e) = instance_store::delete_by_name(&self.ctx, &self.ctx.instance.name).await {
                warn!(target: "persistence", "could not delete instance: {}", e)
            }

        // Create the instance
        if let Err(e) = instance_store::insert(&self.ctx, self.ctx.instance.clone()).await {
            error!(target: "persistence", "could not create instance: {}", e)
        }

        // Create tables if not exist
        if let Err(e) = tick_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create ticks table: {}", e)
        }
        if let Err(e) = trade_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create trades table: {}", e)
        }
        if let Err(e) = insight_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create insights table: {}", e)
        }
        if let Err(e) = metric_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create metrics table: {}", e)
        }
        info!(target: "persistence", "service setup complete");
    }

    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(flush_task(self.clone(), service_ctx.clone(), core_ctx.clone()))]
    }

    async fn teardown(&self, service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        info!(target: "persistence", "service teardown...");
        info!(target: "persistence", "flushing persistence service on teardown...");
        self.flush_all(service_ctx).await;
        self.ctx.pg_pool.close().await;
    }
}
