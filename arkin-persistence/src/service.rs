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
    pub async fn new(
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

    pub async fn get_instance_by_name(&self, name: &str) -> Result<Arc<Instance>, PersistenceError> {
        instance_store::read_by_name(&self.ctx, name).await
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        pipeline_store::read_by_id(&self.ctx, &id).await // Assume impl added
    }

    pub async fn get_strategy(&self, id: Uuid) -> Result<Arc<Strategy>, PersistenceError> {
        strategy_store::read_by_id(&self.ctx, &id).await // Assume impl added
    }

    pub async fn get_instrument(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        instrument_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn get_venue(&self, id: Uuid) -> Result<Arc<Venue>, PersistenceError> {
        venue_store::read_by_id(&self.ctx, &id).await
    }

    pub async fn get_asset(&self, id: Uuid) -> Result<Arc<Asset>, PersistenceError> {
        asset_store::read_by_id(&self.ctx, &id).await
    }

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

    pub async fn trade_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> impl Stream<Item = Arc<Trade>> + 'static {
        trade_store::stream_range_buffered(&self.ctx, instruments, start, end, buffer_size, frequency).await
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn flush_all(&self, ctx: Arc<ServiceCtx>) {
        info!("flushing...");
        let insights = {
            let mut lock = self.ctx.buffer.insights.lock().await;
            let insights = std::mem::take(&mut *lock);
            info!("buffer length {}", lock.len());
            insights
        };

        if insights.is_empty() {
            debug!("No insights to flush.");
            return;
        }

        let insights = insights.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        let persistence_ctx = self.ctx.clone();
        ctx.spawn(async move {
            debug!("Flushing {} insights", insights.len());

            // Insert the insights into the database
            loop {
                match insight_store::insert_vec(&persistence_ctx, &insights).await {
                    Ok(_) => {
                        info!("Successfully flushed {} insights", insights.len());
                        break;
                    }
                    Err(e) => {
                        error!("Failed to flush insights: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
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
            Event::TickUpdate(tick) => {
                if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
                    let mut lock = self.ctx.buffer.ticks.lock().await;
                    lock.push(tick);
                    drop(lock);

                    // Needs to be called on flush
                    // tick_store::insert_vec(&self.ctx, tick).await.expect("Handeled by the module");
                }
            }
            Event::TradeUpdate(trade) => {
                // Persist only if not in Live mode
                if self.mode == InstanceType::Live || self.mode == InstanceType::Utility {
                    let mut lock = self.ctx.buffer.trades.lock().await;
                    lock.push(trade);
                    drop(lock);

                    // Needs to be called on flush
                    // trade_store::insert_vec(&self.ctx, trade).await.expect("Handeled by the module");
                }
            }
            Event::InsightsUpdate(tick) => {
                // Filter out non normalized insights from insight tick
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
                drop(lock);

                // Needs to be called on flush
                // insight_store::insert_vec(&self.ctx, insights).await;
            }
            Event::AccountNew(a) => {
                let _ = account_store::insert(&self.ctx, a).await;
            }
            Event::TransferNew(t) => {
                let _ = transfer_store::insert_batch(&self.ctx, t).await;
            }
            e => warn!(target: "persistence", "received unused event {}", e.event_type()),
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
