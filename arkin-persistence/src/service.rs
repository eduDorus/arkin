use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use arkin_core::prelude::*;

use crate::repos::*;
use crate::stores::*;
use crate::traits::Persistor;
use crate::{PersistenceConfig, PersistenceError};

#[derive(Debug)]
pub struct PersistenceService {
    pub pubsub: Arc<PubSub>,
    pub auto_commit_interval: Duration,
    pub instance_store: InstanceStore,
    pub portfolio_store: PortfolioStore,
    pub transaction_store: TransactionStore,
    pub venue_store: VenueStore,
    pub asset_store: AssetStore,
    pub instrument_store: InstrumentStore,
    pub pipeline_store: PipelineStore,
    pub insights_store: InsightsStore,
    pub strategy_store: StrategyStore,
    pub signal_store: SignalStore,
    pub allocation_store: AllocationStore,
    pub execution_order_store: ExecutionOrderStore,
    pub venue_order_store: VenueOrderStore,
    pub tick_store: TickStore,
    pub trade_store: TradeStore,
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
        let instance_repo = InstanceRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build VenueRepo");
        let portfolio = PortfolioRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build PortfolioRepo");
        let transactions_repo = TransactionRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build TransactionRepo");
        let venue_repo = VenueRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build VenueRepo");
        let asset_repo = AssetRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build AssetRepo");
        let instrument_repo = InstrumentRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build InstrumentRepo");
        let pipeline_repo = PipelineRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build PipelineRepo");
        let insights_repo = InsightsRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build InsightsRepo");
        let strategy_repo = StrategyRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build InsightsRepo");
        let signal_repo = SignalRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build SignalRepo");
        let allocation_repo = AllocationRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build AllocationRepo");
        let execution_order_repo = ExecutionOrderRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build ExecutionOrderRepo");
        let venue_order_repo = VenueOrderRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build TransactionsRepo");
        let tick_repo = TickRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build TickRepo");
        let trade_repo = TradeRepoBuilder::default()
            .pool(pool.clone())
            .build()
            .expect("Failed to build TradeRepo");

        // Initialize stores
        let instance_store = InstanceStoreBuilder::default()
            .instance_repo(instance_repo.clone())
            .build()
            .expect("Failed to build InstanceStore");
        let portfolio_store = PortfolioStoreBuilder::default()
            .portfolio_repo(portfolio.clone())
            .build()
            .expect("Failed to build PortfolioStore");
        let transaction_store = TransactionStoreBuilder::default()
            .transaction_repo(transactions_repo.clone())
            .build()
            .expect("Failed to build TransactionStore");
        let venue_store = VenueStoreBuilder::default()
            .venue_repo(venue_repo)
            .build()
            .expect("Failed to build VenueStore");
        let asset_store = AssetStoreBuilder::default()
            .asset_repo(asset_repo.clone())
            .build()
            .expect("Failed to build AssetStore");
        let instrument_store = InstrumentStoreBuilder::default()
            .instrument_repo(instrument_repo)
            .asset_store(asset_store.clone())
            .venue_store(venue_store.clone())
            .build()
            .expect("Failed to build InstrumentStore");
        let pipeline_store = PipelineStoreBuilder::default()
            .pipeline_repo(pipeline_repo.clone())
            .build()
            .expect("Failed to build PipelineStore");
        let insights_store = InsightsStoreBuilder::default()
            .insights_repo(insights_repo.clone())
            .build()
            .expect("Failed to build InsightsStore");
        let strategy_store = StrategyStoreBuilder::default()
            .strategy_repo(strategy_repo.clone())
            .build()
            .expect("Failed to build InsightsStore");
        let signal_store = SignalStoreBuilder::default()
            .signal_repo(signal_repo.clone())
            .build()
            .expect("Failed to build SignalStore");
        let allocation_store = AllocationStoreBuilder::default()
            .allocation_repo(allocation_repo.clone())
            .build()
            .expect("Failed to build AllocationStore");
        let execution_order_store = ExecutionOrderStoreBuilder::default()
            .execution_order_repo(execution_order_repo.clone())
            .build()
            .expect("Failed to build ExecutionOrderStore");
        let venue_order_store = VenueOrderStoreBuilder::default()
            .venue_order_repo(venue_order_repo.clone())
            .build()
            .expect("Failed to build VenueOrderStore");
        let tick_store = TickStoreBuilder::default()
            .tick_repo(tick_repo)
            .instrument_store(instrument_store.clone())
            .build()
            .expect("Failed to build TickStore");
        let trade_store = TradeStoreBuilder::default()
            .trade_repo(trade_repo)
            .instrument_store(instrument_store.clone())
            .build()
            .expect("Failed to build TradeStore");

        Self {
            pubsub,
            auto_commit_interval: Duration::from_secs(config.auto_commit_interval),
            instance_store,
            portfolio_store,
            transaction_store,
            venue_store,
            asset_store,
            instrument_store,
            pipeline_store,
            insights_store,
            strategy_store,
            signal_store,
            allocation_store,
            execution_order_store,
            venue_order_store,
            tick_store,
            trade_store,
        }
    }
}

#[async_trait]
impl Persistor for PersistenceService {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PersistenceError> {
        info!("Starting persistence service...");

        let mut interval = tokio::time::interval(self.auto_commit_interval);

        let mut trades = self.pubsub.subscribe::<Trade>();
        let mut ticks = self.pubsub.subscribe::<Tick>();
        let mut insights = self.pubsub.subscribe::<Insight>();
        let mut insights_tick = self.pubsub.subscribe::<InsightTick>();

        loop {
            tokio::select! {
                    Ok(trade) = trades.recv() => {
                        if let Err(e) = self.trade_store.insert_buffered(trade).await {
                            error!("Failed to insert trade: {}", e);
                        }
                    }
                    Ok(tick) = ticks.recv() => {
                        if let Err(e) = self.tick_store.insert_buffered(tick).await {
                            error!("Failed to insert tick: {}", e);
                        }
                    }
                    Ok(insight) = insights.recv() => {
                        if let Err(e) = self.insights_store.insert_buffered(insight).await {
                            error!("Failed to insert insight: {}", e);
                        }
                    }
                    Ok(insight_tick) = insights_tick.recv() => {
                        if let Err(e) = self.insights_store.insert_buffered_vec(insight_tick.insights).await {
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
        self.tick_store.flush().await?;
        self.trade_store.flush().await?;
        self.insights_store.flush().await?;
        Ok(())
    }
}
