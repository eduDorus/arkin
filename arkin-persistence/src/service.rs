use std::pin::Pin;
use std::str::FromStr;
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

macro_rules! flush_buffer {
    ($service_ctx:expr, $persistence_ctx:expr, $buffer:expr, $name:expr, $insert:expr) => {
        let data = {
            let mut lock = $buffer.lock().await;
            std::mem::take(&mut *lock)
        };
        debug!(target: "persistence", "{} buffer length {}", $name, data.len());

        if !data.is_empty() {
            let persistence_ctx = $persistence_ctx.clone();
            $service_ctx.spawn(async move {
                debug!(target: "persistence", "flushing {} {}", data.len(), $name);

                loop {
                    match $insert(&persistence_ctx, &data).await {
                        Ok(_) => {
                            debug!(target: "persistence", "successfully flushed {} {}", data.len(), $name);
                            break;
                        }
                        Err(e) => {
                            error!(target: "persistence", "failed to flush {}: {}", $name, e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    };
}

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
            .with_option("buffer_size", "16777216")
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

    pub fn from_config(instance: Instance, only_normalized: bool, only_predictions: bool, dry_run: bool) -> Arc<Self> {
        let config = load::<PersistenceConfig>();
        Self::new(&config, instance, only_normalized, only_predictions, dry_run)
    }

    pub fn from_config_test() -> Arc<Self> {
        let config = load::<PersistenceConfig>();
        let now: UtcDateTime = time::OffsetDateTime::now_utc().into();
        let instance = Instance::builder()
            .id(Uuid::from_str("04432ac5-483d-46a3-811b-6be79d6ef7c1").expect("Failed to parse test instance UUID"))
            .name("test-instance".to_string())
            .instance_type(InstanceType::Test)
            .created(now)
            .updated(now)
            .build();
        Self::new(&config, instance, false, false, true)
    }

    pub async fn refresh(&self) -> Result<(), PersistenceError> {
        info!(target: "persistence", "refreshing cache...");

        // Load all instances into cache
        let instances_loaded = instance_store::load_instances(&self.ctx).await?;
        info!(target: "persistence", "loaded {} instances into cache", instances_loaded.len());

        // Load all venues into cache (must be first as instruments depend on venues)
        let venues_loaded = venue_store::load_venues(&self.ctx).await?;
        info!(target: "persistence", "loaded {} venues into cache", venues_loaded.len());

        // Load all assets into cache (must be before instruments as instruments depend on assets)
        let assets_loaded = asset_store::load_assets(&self.ctx).await?;
        info!(target: "persistence", "loaded {} assets into cache", assets_loaded.len());

        // Load all instruments into cache
        let instruments_loaded = instrument_store::load_instruments(&self.ctx).await?;
        info!(target: "persistence", "loaded {} instruments into cache", instruments_loaded.len());

        // Load all accounts into cache
        let accounts_loaded = account_store::load_accounts(&self.ctx).await?;
        info!(target: "persistence", "loaded {} accounts into cache", accounts_loaded.len());

        // Load all strategies into cache
        let strategies_loaded = strategy_store::load_strategies(&self.ctx).await?;
        info!(target: "persistence", "loaded {} strategies into cache", strategies_loaded.len());

        Ok(())
    }

    // pub async fn get_pipeline(&self, id: Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
    //     pipeline_store::read_by_id(&self.ctx, &id).await
    // }

    // pub async fn get_pipeline_by_name(&self, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
    //     pipeline_store::read_by_name(&self.ctx, name).await
    // }

    pub async fn insert_pipeline(&self, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
        pipeline_store::insert(&self.ctx, pipeline).await
    }

    // pub async fn get_strategy(&self, id: Uuid) -> Result<Arc<Strategy>, PersistenceError> {
    //     strategy_store::read_by_id(&self.ctx, &id).await
    // }

    // pub async fn get_instrument(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError> {
    //     instrument_store::read_by_id(&self.ctx, &id).await
    // }

    // pub async fn list_instruments_by_venue_symbol(
    //     &self,
    //     symbols: &[String],
    //     venue: &Arc<Venue>,
    // ) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
    //     let mut instruments = Vec::with_capacity(symbols.len());
    //     for symbol in symbols {
    //         let inst = instrument_store::read_by_venue_symbol(&self.ctx, symbol, venue).await?;
    //         instruments.push(inst);
    //     }
    //     Ok(instruments)
    // }

    // pub async fn get_asset(&self, id: Uuid) -> Result<Arc<Asset>, PersistenceError> {
    //     asset_store::read_by_id(&self.ctx, &id).await
    // }

    // pub async fn get_scaler_data(
    //     &self,
    //     pipeline: &Arc<Pipeline>,
    //     instrument: &Arc<Instrument>,
    //     from: UtcDateTime,
    //     till: UtcDateTime,
    //     levels: &[f64],
    // ) -> Result<Vec<QuantileData>, PersistenceError> {
    //     scaler_store::get_iqr(&self.ctx, pipeline, instrument, from, till, levels).await
    // }

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
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.execution_orders.lock().await;
            lock.push(order);
        }
    }

    pub async fn insert_venue_order(&self, order: Arc<VenueOrder>) {
        if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
            let mut lock = self.ctx.buffer.venue_orders.lock().await;
            lock.push(order);
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

    pub async fn insert_audit(&self, event: Event) {
        if self.mode != InstanceType::Test {
            let mut lock = self.ctx.buffer.audits.lock().await;
            lock.push(event);
        }
    }

    // TODO: WE NEED TO FLUSH ALSO TRADES AND TICKS
    pub async fn flush_all(&self, ctx: Arc<ServiceCtx>) {
        debug!(target: "persistence", "flushing...");

        flush_buffer!(ctx, self.ctx, self.ctx.buffer.insights, "insights", |ctx, data| {
            insight_store::insert_vec(ctx, data)
        });
        flush_buffer!(ctx, self.ctx, self.ctx.buffer.trades, "trades", |ctx, data| {
            trade_store::insert_vec(ctx, data)
        });
        flush_buffer!(ctx, self.ctx, self.ctx.buffer.ticks, "ticks", |ctx, data| {
            tick_store::insert_vec(ctx, data)
        });
        flush_buffer!(ctx, self.ctx, self.ctx.buffer.metrics, "metrics", |ctx, data| {
            metric_store::batch_insert_metric(ctx, data)
        });
        flush_buffer!(
            ctx,
            self.ctx,
            self.ctx.buffer.execution_orders,
            "execution orders",
            |ctx, data| execution_order_store::insert_batch(ctx, data)
        );
        flush_buffer!(ctx, self.ctx, self.ctx.buffer.venue_orders, "venue orders", |ctx, data| {
            venue_order_store::insert_batch(ctx, data)
        });
        flush_buffer!(ctx, self.ctx, self.ctx.buffer.audits, "audits", |ctx, data| {
            audit_store::insert_batch(ctx, data)
        });
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
    /// Refresh cache by reloading venues, assets, and instruments from database
    async fn refresh(&self) -> Result<(), PersistenceError> {
        info!(target: "persistence", "refreshing cache...");

        // Load all venues into cache (must be first as instruments depend on venues)
        let venues_loaded = venue_store::load_venues(&self.ctx).await?;
        info!(target: "persistence", "loaded {} venues into cache", venues_loaded.len());

        // Load all assets into cache (must be before instruments as instruments depend on assets)
        let assets_loaded = asset_store::load_assets(&self.ctx).await?;
        info!(target: "persistence", "loaded {} assets into cache", assets_loaded.len());

        // Load all instruments into cache
        let instruments_loaded = instrument_store::load_instruments(&self.ctx).await?;
        info!(target: "persistence", "loaded {} instruments into cache", instruments_loaded.len());

        // Load all accounts into cache
        let accounts_loaded = account_store::load_accounts(&self.ctx).await?;
        info!(target: "persistence", "loaded {} accounts into cache", accounts_loaded.len());

        // Load all strategies into cache
        let strategies_loaded = strategy_store::load_strategies(&self.ctx).await?;
        info!(target: "persistence", "loaded {} strategies into cache", strategies_loaded.len());

        Ok(())
    }

    async fn list_instances(&self, query: &InstanceListQuery) -> Result<Vec<Arc<Instance>>, PersistenceError> {
        instance_store::query_list(&self.ctx, query).await
    }

    async fn list_pipelines(&self, query: &PipelineListQuery) -> Result<Vec<Arc<Pipeline>>, PersistenceError> {
        pipeline_store::query_list(&self.ctx, query).await
    }

    async fn list_venues(&self, query: &VenueListQuery) -> Result<Vec<Arc<Venue>>, PersistenceError> {
        venue_store::query_list(&self.ctx, query).await
    }

    async fn list_assets(&self, query: &AssetListQuery) -> Result<Vec<Arc<Asset>>, PersistenceError> {
        asset_store::query_list(&self.ctx, query).await
    }

    async fn list_instruments(&self, query: &InstrumentListQuery) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        instrument_store::query_list(&self.ctx, query).await
    }

    async fn get_instrument(&self, query: &InstrumentQuery) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::query(&self.ctx, query).await
    }

    async fn get_instance(&self, query: &InstanceQuery) -> Result<Arc<Instance>, PersistenceError> {
        instance_store::query(&self.ctx, query).await
    }

    async fn get_pipeline(&self, query: &PipelineQuery) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::query(&self.ctx, query).await
    }

    async fn get_venue(&self, query: &VenueQuery) -> Result<Arc<Venue>, PersistenceError> {
        venue_store::query(&self.ctx, query).await
    }

    async fn get_asset(&self, query: &AssetQuery) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::query(&self.ctx, query).await
    }

    async fn get_feature(&self, query: &FeatureQuery) -> FeatureId {
        feature_store::read_feature_id(&self.ctx, &query.id).await
    }

    async fn get_account(&self, query: &AccountQuery) -> Result<Arc<Account>, PersistenceError> {
        account_store::query(&self.ctx, query).await
    }

    async fn get_strategy(&self, query: &StrategyQuery) -> Result<Arc<Strategy>, PersistenceError> {
        strategy_store::query(&self.ctx, query).await
    }

    async fn list_accounts(&self, query: &AccountListQuery) -> Result<Vec<Arc<Account>>, PersistenceError> {
        account_store::query_list(&self.ctx, query).await
    }

    async fn list_strategies(&self, query: &StrategyListQuery) -> Result<Vec<Arc<Strategy>>, PersistenceError> {
        strategy_store::query_list(&self.ctx, query).await
    }

    async fn list_features(&self, query: &FeatureListQuery) -> Result<Vec<FeatureId>, PersistenceError> {
        Ok(feature_store::list_features(&self.ctx, query).await)
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
        // trade_store::stream_range_full(&self.ctx, instruments, start, end, buffer_size, frequency).await
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
    fn event_filter(&self, instance_type: InstanceType) -> EventFilter {
        match instance_type {
            InstanceType::Live | InstanceType::Utility => EventFilter::All,
            InstanceType::Simulation => EventFilter::Events(vec![
                EventType::InsightsUpdate,
                EventType::MetricUpdate,
                EventType::NewAccount,
                EventType::NewTransfer,
                EventType::NewTransferBatch,
                EventType::ExecutionOrderBookUpdate,
                EventType::VenueOrderBookUpdate,
            ]),
            InstanceType::Insights => EventFilter::Events(vec![EventType::InsightsUpdate]),
            InstanceType::Test => EventFilter::None,
        }
    }

    async fn handle_event(&self, _ctx: Arc<CoreCtx>, event: Event) {
        if self.dry_run {
            return;
        }

        if !event.event_type().is_market_data() {
            self.insert_audit(event.clone()).await;
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
            Event::ExecutionOrderBookUpdate(o) => self.insert_execution_order(o).await,

            // Venue Orders
            Event::VenueOrderBookUpdate(o) => self.insert_venue_order(o).await,
            _ => {}
        }
    }

    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // TODO: NOT FOR PRODUCTION
        if self.ctx.instance.instance_type == InstanceType::Test
            && let Err(e) = instance_store::delete_by_id(&self.ctx, self.ctx.instance.id).await
        {
            warn!(target: "persistence", "could not delete instance: {}", e)
        }

        if self.ctx.instance.instance_type == InstanceType::Simulation
            && let Err(e) = instance_store::delete_by_name(&self.ctx, &self.ctx.instance.name).await
        {
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
        if let Err(e) = execution_order_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create execution_orders table: {}", e)
        }
        if let Err(e) = venue_order_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create venue_orders table: {}", e)
        }
        if let Err(e) = audit_store::create_table(&self.ctx).await {
            error!(target: "persistence", "could not create audit table: {}", e)
        }

        // Populate cache with instruments and assets
        if let Err(e) = self.refresh().await {
            error!(target: "persistence", "could not populate cache: {}", e)
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
