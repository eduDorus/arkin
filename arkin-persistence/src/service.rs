use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::ConnectOptions;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{error, info};

use arkin_core::prelude::*;

use crate::repos::*;
use crate::stores::*;
use crate::{PersistenceConfig, PersistenceError};

pub struct PersistenceService {
    pub pubsub: PubSubSubscriber,
    pub mode: InstanceType,
    pub dry_run: bool,
    pub only_normalized: bool,
    pub only_predictions: bool,
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
    pub async fn new(
        pubsub: PubSubSubscriber,
        config: &PersistenceConfig,
        instance: Instance,
        only_normalized: bool,
        only_predictions: bool,
        dry_run: bool,
    ) -> Arc<Self> {
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
            mode: instance.instance_type,
            dry_run,
            only_normalized,
            only_predictions,
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

        let (event_buffer_tx, event_buffer_rx) = kanal::bounded_async(100000);

        // Spawn a task to persist events
        let mode = self.mode.clone();
        let tick_store = self.tick_store.clone();
        let trade_store = self.trade_store.clone();
        let insights_store = self.insights_store.clone();
        let signal_store = self.signal_store.clone();
        let execution_order_store = self.execution_order_store.clone();
        let venue_order_store = self.venue_order_store.clone();
        let account_store = self.account_store.clone();
        let transfer_store = self.transfer_store.clone();
        let task_tracker = TaskTracker::new();
        let shutdown_clone = shutdown.clone();
        task_tracker.spawn(async move {
            loop {
                if event_buffer_rx.is_empty() && shutdown_clone.is_cancelled() {
                    info!("Event buffer is empty and shutdown is cancelled");
                    break;
                }
                match event_buffer_rx.try_recv() {
                    Ok(Some(event)) => {
                        let res = match event {
                            Event::TickUpdate(tick) => {
                                // Persist only if not in Live mode
                                if mode != InstanceType::Live || mode != InstanceType::Utility {
                                    tick_store.insert_buffered(tick).await
                                } else {
                                    Ok(())
                                }
                            }
                            Event::TradeUpdate(trade) => {
                                // Persist only if not in Live mode
                                if mode != InstanceType::Live || mode != InstanceType::Utility {
                                    trade_store.insert_buffered(trade).await
                                } else {
                                    Ok(())
                                }
                            }
                            Event::InsightsUpdate(tick) => {
                                insights_store.insert_buffered_vec(tick.insights.clone()).await
                            }
                            Event::SignalUpdate(signal) => signal_store.insert(signal).await,
                            Event::ExecutionOrderNew(order) => execution_order_store.insert(order).await,
                            Event::ExecutionOrderStatusUpdate(order) => execution_order_store.update(order).await,
                            Event::VenueOrderNew(order) => venue_order_store.insert(order).await,
                            Event::VenueOrderFillUpdate(order) => venue_order_store.update(order).await,
                            Event::AccountNew(account) => account_store.insert(account).await,
                            Event::TransferNew(transfer) => transfer_store.insert_batch(transfer).await,
                            _ => Ok(()),
                        };
                        if let Err(e) = res {
                            error!("Failed to persist event: {:?}", e);
                        }
                    }
                    Ok(None) => {
                        // No event to process
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    Err(e) => match e {
                        kanal::ReceiveError::SendClosed => {
                            info!("Event buffer closed");
                            // Print if there are any events left in the buffer
                            if event_buffer_rx.len() > 0 {
                                info!("Remaining events in buffer: {}", event_buffer_rx.len());
                            }
                            break;
                        }
                        kanal::ReceiveError::Closed => {
                            info!("Event buffer closed");
                            if event_buffer_rx.len() > 0 {
                                info!("Remaining events in buffer: {}", event_buffer_rx.len());
                            }
                            break;
                        }
                    },
                }
            }
            if let Err(e) = tick_store.flush().await {
                error!("Failed to flush tick store: {:?}", e);
            }
            if let Err(e) = trade_store.flush().await {
                error!("Failed to flush trade store: {:?}", e);
            }
            if let Err(e) = insights_store.flush().await {
                error!("Failed to flush insights store: {:?}", e);
            }
            info!("Persistence service event buffer task stopped.");
        });

        loop {
            tokio::select! {
                    Some(mut event) = self.pubsub.recv() => {
                        // If we do a dry run we don't save any data
                        if self.dry_run {
                            self.pubsub.ack().await;
                            continue;
                        }

                        // Filter out non normalized insights from insight tick
                        if self.only_normalized {
                            if let Event::InsightsUpdate(t) = event {
                                let insights = t.insights.iter().filter(|i| i.insight_type == InsightType::Normalized).cloned().collect::<Vec<_>>();
                                event = Event::InsightsUpdate(InsightsUpdate::builder().event_time(t.event_time).instruments(t.instruments.clone()).insights(insights).build().into());
                            }
                        }

                        // Filter out non prediction insights from insight tick
                        if self.only_predictions {
                            if let Event::InsightsUpdate(t) = event {
                                let insights = t.insights.iter().filter(|i| i.insight_type == InsightType::Prediction).cloned().collect::<Vec<_>>();
                                event = Event::InsightsUpdate(InsightsUpdate::builder().event_time(t.event_time).instruments(t.instruments.clone()).insights(insights).build().into());
                            }
                        }

                        if let Err(e) = event_buffer_tx.send(event).await {
                            error!("Failed to send event to persistence buffer: {:?}", e);
                        }

                        self.pubsub.ack().await;
                    }
                    _ = shutdown.cancelled() => {
                        info!("Persistence service shutdown...");
                        task_tracker.close();
                        task_tracker.wait().await;
                        if let Err(e) = self.flush().await {
                            error!("Failed to commit persistence service on shutdown: {}", e);
                        }
                        if let Err(e) = self.close().await {
                            error!("Failed to close persistence service on shutdown: {}", e);
                        }
                        break;
                    }
            }
        }
        info!("Persistence service stopped.");
        Ok(())
    }
}
