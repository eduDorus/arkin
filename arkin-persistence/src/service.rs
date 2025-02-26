use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::ConnectOptions;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use arkin_core::prelude::*;

use crate::repos::*;
use crate::stores::*;
use crate::{PersistenceConfig, PersistenceError};

#[derive(Debug)]
pub struct PersistenceService {
    pub pubsub: Arc<PubSub>,
    pub dry_run: bool,
    pub auto_commit_interval: Duration,
    pub instance_store: Arc<InstanceStore>,
    pub account_store: Arc<AccountStore>,
    pub transfer_store: Arc<TransferStore>,
    pub venue_store: Arc<VenueStore>,
    pub asset_store: Arc<AssetStore>,
    pub instrument_store: Arc<InstrumentStore>,
    pub pipeline_store: Arc<PipelineStore>,
    pub insights_store: Arc<InsightsStore>,
    pub strategy_store: Arc<StrategyStore>,
    pub signal_store: Arc<SignalStore>,
    // pub allocation_store: Arc<AllocationStore>,
    pub execution_order_store: Arc<ExecutionOrderStore>,
    pub venue_order_store: Arc<VenueOrderStore>,
    pub tick_store: Arc<TickStore>,
    pub trade_store: Arc<TradeStore>,
}

impl PersistenceService {
    pub async fn new(pubsub: Arc<PubSub>, config: &PersistenceConfig, instance: Instance, dry_run: bool) -> Arc<Self> {
        let db_config = config.database.clone();
        let conn_options = PgConnectOptions::new()
            .host(&db_config.host)
            .port(db_config.port)
            .username(&db_config.user)
            .password(&db_config.password)
            .database(&db_config.database)
            .ssl_mode(PgSslMode::Prefer)
            .log_statements("DEBUG".parse().unwrap())
            .log_slow_statements("DEBUG".parse().unwrap(), Duration::from_secs(300));

        let pool = PgPoolOptions::new()
            .min_connections(db_config.min_connections)
            .max_connections(db_config.max_connections)
            .idle_timeout(Duration::from_secs(db_config.idle_timeout))
            .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
            .max_lifetime(Duration::from_secs(db_config.max_lifetime))
            .connect_lazy_with(conn_options);

        // Initialize repositories
        let instance_repo = InstanceRepo::builder().pool(pool.clone()).build();
        let instance_store = Arc::new(InstanceStore::builder().instance_repo(instance_repo.to_owned()).build());
        let instance = if let Ok(instance) = instance_store.read_by_name(&instance.name).await {
            instance
        } else {
            let instance = Arc::new(instance);
            instance_store.insert(instance.clone()).await.unwrap();
            instance
        };

        let account_repo = AccountRepo::builder().pool(pool.clone()).instance(instance.clone()).build();
        let transfer_repo = TransferRepo::builder().pool(pool.clone()).instance(instance.clone()).build();
        let venue_repo = VenueRepo::builder().pool(pool.clone()).build();
        let asset_repo = AssetRepo::builder().pool(pool.clone()).build();
        let instrument_repo = InstrumentRepo::builder().pool(pool.clone()).build();
        let pipeline_repo = PipelineRepo::builder().pool(pool.clone()).build();
        // let insights_repo = InsightsParquetRepo::new("insights_latest.parquet").await.unwrap();
        // let insights_repo = InsightsRepo::builder().pool(pool.clone()).build();
        let insights_repo = InsightsClickhouseRepo::new();
        insights_repo.create_table().await.unwrap();
        let strategy_repo = StrategyRepo::builder().pool(pool.clone()).build();
        let signal_repo = SignalRepo::builder().pool(pool.clone()).instance(instance.clone()).build();
        // let allocation_repo = AllocationRepo::builder().pool(pool.clone()).build();
        let execution_order_repo = ExecutionOrderRepo::builder()
            .pool(pool.clone())
            .instance(instance.clone())
            .build();
        let venue_order_repo = VenueOrderRepo::builder().pool(pool.clone()).instance(instance.clone()).build();

        let tick_repo = TickClickhouseRepo::new();
        tick_repo.create_table().await.unwrap();
        let trade_repo = TradeClickhouseRepo::new();
        trade_repo.create_table().await.unwrap();
        // let tick_repo = TickRepo::builder().pool(pool.clone()).build();
        // let trade_repo = TradeRepo::builder().pool(pool.clone()).build();
        // let trade_repo = TradeParquetRepo::new().await.unwrap();

        // Initialize stores

        let account_store = Arc::new(AccountStore::builder().account_repo(account_repo.to_owned()).build());
        let transfer_store = Arc::new(TransferStore::builder().transfer_repo(transfer_repo.to_owned()).build());
        let venue_store = Arc::new(VenueStore::builder().venue_repo(venue_repo).build());
        let asset_store = Arc::new(AssetStore::builder().asset_repo(asset_repo.to_owned()).build());
        let instrument_store = Arc::new(
            InstrumentStore::builder()
                .instrument_repo(instrument_repo)
                .asset_store(asset_store.to_owned())
                .venue_store(venue_store.to_owned())
                .build(),
        );
        let pipeline_store = Arc::new(PipelineStore::builder().pipeline_repo(pipeline_repo.to_owned()).build());
        let insights_store = Arc::new(
            InsightsStore::builder()
                .insights_repo(insights_repo.to_owned())
                .buffer_size(config.batch_size)
                .build(),
        );
        let strategy_store = Arc::new(StrategyStore::builder().strategy_repo(strategy_repo.to_owned()).build());
        let signal_store = Arc::new(SignalStore::builder().signal_repo(signal_repo.to_owned()).build());
        // let allocation_store = Arc::new(AllocationStore::builder().allocation_repo(allocation_repo.to_owned()).build());
        let execution_order_store = Arc::new(
            ExecutionOrderStore::builder()
                .execution_order_repo(execution_order_repo.to_owned())
                .build(),
        );
        let venue_order_store =
            Arc::new(VenueOrderStore::builder().venue_order_repo(venue_order_repo.to_owned()).build());
        let tick_store = Arc::new(
            TickStore::builder()
                .tick_repo(tick_repo.into())
                .instrument_store(instrument_store.to_owned())
                .buffer_size(config.batch_size)
                .build(),
        );
        let trade_store = Arc::new(
            TradeStore::builder()
                .trade_repo(trade_repo.into())
                .instrument_store(instrument_store.to_owned())
                .buffer_size(config.batch_size)
                .build(),
        );

        let service = Self {
            pubsub,
            dry_run,
            auto_commit_interval: Duration::from_secs(config.auto_commit_interval),
            instance_store,
            account_store,
            transfer_store,
            venue_store,
            asset_store,
            instrument_store,
            pipeline_store,
            insights_store,
            strategy_store,
            signal_store,
            // allocation_store,
            execution_order_store,
            venue_order_store,
            tick_store,
            trade_store,
        };
        Arc::new(service)
    }

    pub async fn flush(&self) -> Result<(), PersistenceError> {
        self.tick_store.flush().await?;
        self.trade_store.flush().await?;
        self.insights_store.flush().await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), PersistenceError> {
        self.insights_store.close().await?;
        Ok(())
    }
}

#[async_trait]
impl RunnableService for PersistenceService {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting persistence service...");

        let mut interval = tokio::time::interval(self.auto_commit_interval);
        let mut rx = self.pubsub.subscribe();

        loop {
            tokio::select! {
                    Ok(event) = rx.recv() => {
                        if self.dry_run {
                          // Still need to update price cache
                            if let Event::Tick(t) = event {
                              self.tick_store.update_tick_cache(t).await;
                            }
                            continue;
                        }
                        let res = match event {
                            Event::Tick(tick) => self.tick_store.insert_buffered(tick).await,
                            Event::Trade(trade) =>self.trade_store.insert_buffered(trade).await,
                            Event::Insight(insight) => self.insights_store.insert_buffered(insight).await,
                            Event::InsightTick(tick) => self.insights_store.insert_buffered_vec(tick.insights.clone()).await,
                            Event::Signal(signal) => self.signal_store.insert(signal).await,
                            Event::ExecutionOrder(order) => self.execution_order_store.insert(order).await ,
                            Event::ExecutionOrderUpdate(order) => self.execution_order_store.update(order).await,
                            Event::VenueOrderNew(order) => self.venue_order_store.insert(order).await,
                            Event::VenueOrderFillUpdate(order) => self.venue_order_store.update(order).await,
                            Event::AccountNew(account) => self.account_store.insert(account).await,
                            Event::Transfer(transfer) => self.transfer_store.insert(transfer).await,
                            Event::TransferBatch(transfers) => self.transfer_store.insert_batch(transfers).await,
                            _ => {Ok(())}
                        };
                        if let Err(e) = res {
                            error!("Failed to insert event: {:?}", e);
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
                        if let Err(e) = self.close().await {
                            error!("Failed to close persistence service on shutdown: {}", e);
                        }
                        // wait 5 seconds
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        break;
                    }
            }
        }

        info!("Persistence service shutdown...");
        Ok(())
    }
}
