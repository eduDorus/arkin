use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use typed_builder::TypedBuilder;

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_execution::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;
use arkin_strategies::prelude::*;

use crate::{TradingEngine, TradingEngineError};

#[derive(Debug, TypedBuilder)]
pub struct SingleStrategyEngine {
    pubsub: Arc<PubSub>,
    instruments: Vec<Arc<Instrument>>,

    #[builder(default)]
    persistor_task_tracker: TaskTracker,
    #[builder(default)]
    persistor_shutdown: CancellationToken,
    persistor: Arc<dyn Persistor>,

    #[builder(default)]
    portfolio_task_tracker: TaskTracker,
    #[builder(default)]
    portfolio_shutdown: CancellationToken,
    portfolio: Arc<dyn Accounting>,

    #[builder(default)]
    ingestor_task_tracker: TaskTracker,
    #[builder(default)]
    ingestor_shutdown: CancellationToken,
    ingestors: Vec<Arc<dyn Ingestor>>,

    #[builder(default)]
    insights_task_tracker: TaskTracker,
    #[builder(default)]
    insights_shutdown: CancellationToken,
    insights: Arc<dyn Insights>,

    #[builder(default)]
    strategy_task_tracker: TaskTracker,
    #[builder(default)]
    strategy_shutdown: CancellationToken,
    strategy: Arc<dyn Algorithm>,

    #[builder(default)]
    allocation_task_tracker: TaskTracker,
    #[builder(default)]
    allocation_shutdown: CancellationToken,
    allocation_optim: Arc<dyn AllocationOptim>,

    #[builder(default)]
    order_manager_task_tracker: TaskTracker,
    #[builder(default)]
    order_manager_shutdown: CancellationToken,
    order_manager: Arc<dyn OrderManager>,

    #[builder(default)]
    executor_tracker: TaskTracker,
    #[builder(default)]
    executor_shutdown: CancellationToken,
    executor: Arc<dyn Executor>,
}

impl SingleStrategyEngine {
    async fn load_state(&self) -> Result<(), TradingEngineError> {
        // Setup Insights
        let start = Instant::now();
        let end_time = OffsetDateTime::now_utc();
        // Round to a minute
        let end_time = end_time.replace_second(0).expect("Failed to replace second");
        let end_time = end_time.replace_nanosecond(0).expect("Failed to replace nanosecond");

        let lookback_data = Duration::from_secs(2 * 3600);
        let lookback_insights = Duration::from_secs(3600);
        self.insights.load(end_time, &self.instruments, lookback_data).await?;
        let mut clock = Clock::new(end_time - lookback_insights, end_time, Duration::from_secs(60));
        while let Some((_start, end)) = clock.next() {
            self.insights.process(end, &self.instruments, false).await?;
        }

        // If we are now at the start of a new minute, we need to load the last minute of data
        let diff = OffsetDateTime::now_utc() - end_time;
        if diff > Duration::from_secs(60) {
            info!("Hopping to the next minute to load the last minute of data");
            self.insights
                .load(end_time + Duration::from_secs(60), &self.instruments, Duration::from_secs(60))
                .await?;
            self.insights
                .process(end_time + Duration::from_secs(60), &self.instruments, false)
                .await?;
        }

        info!("Loaded state in {:?}", start.elapsed());
        Ok(())
    }

    async fn pipeline(&self) -> Result<(), TradingEngineError> {
        let mut time_helper = TickHelper::new(Duration::from_secs(60));

        loop {
            tokio::select! {
                (event_time, frequency) = time_helper.tick() => {
                    info!("Interval tick: {}", event_time);
                    let interval_tick = IntervalTick::builder()
                        .event_time(event_time)
                        .instruments(self.instruments.clone())
                        .frequency(frequency)
                        .build();
                   self.pubsub.publish::<IntervalTick>(interval_tick.into());
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received Ctrl-C, shutting down...");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl TradingEngine for SingleStrategyEngine {
    async fn start(&self) -> Result<(), TradingEngineError> {
        // Start the persistor
        let shutdown = self.persistor_shutdown.clone();
        let persistor = self.persistor.clone();
        self.persistor_task_tracker.spawn(async move {
            if let Err(e) = persistor.start(shutdown).await {
                error!("Error in persistor: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the portfolio
        let shutdown = self.portfolio_shutdown.clone();
        let portfolio = self.portfolio.clone();
        self.portfolio_task_tracker.spawn(async move {
            if let Err(e) = portfolio.start(shutdown).await {
                error!("Error in portfolio: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the ingestors
        for ingestor in &self.ingestors {
            let shutdown = self.ingestor_shutdown.clone();
            let ingestor = ingestor.clone();
            self.ingestor_task_tracker.spawn(async move {
                if let Err(e) = ingestor.start(shutdown).await {
                    error!("Error in ingestor: {}", e);
                }
            });
        }
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the insights
        let shutdown = self.insights_shutdown.clone();
        let insights = self.insights.clone();
        self.insights_task_tracker.spawn(async move {
            if let Err(e) = insights.start(shutdown).await {
                error!("Error in insights: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the strategies
        let shutdown = self.strategy_shutdown.clone();
        let strategy = self.strategy.clone();
        self.strategy_task_tracker.spawn(async move {
            if let Err(e) = strategy.start(shutdown).await {
                error!("Error in strategy: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the allocation optimizer
        let shutdown = self.allocation_shutdown.clone();
        let allocation_optim = self.allocation_optim.clone();
        self.allocation_task_tracker.spawn(async move {
            if let Err(e) = allocation_optim.start(shutdown).await {
                error!("Error in allocation optimizer: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the order manager
        let shutdown = self.order_manager_shutdown.clone();
        let order_manager = self.order_manager.clone();
        self.order_manager_task_tracker.spawn(async move {
            if let Err(e) = order_manager.start(shutdown).await {
                error!("Error in order manager: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Start the executor
        let shutdown = self.executor_shutdown.clone();
        let executor = self.executor.clone();
        self.executor_tracker.spawn(async move {
            if let Err(e) = executor.start(shutdown).await {
                error!("Error in executor: {}", e);
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Load the state
        self.load_state().await?;

        // Run the pipeline
        self.pipeline().await?;

        Ok(())
    }

    async fn stop(&self) -> Result<(), TradingEngineError> {
        info!("Stopping ingestors...");
        self.ingestor_shutdown.cancel();
        self.ingestor_task_tracker.close();
        self.ingestor_task_tracker.wait().await;

        info!("Stopping insights...");
        self.insights_shutdown.cancel();
        self.insights_task_tracker.close();
        self.insights_task_tracker.wait().await;

        info!("Stopping strategies...");
        self.strategy_shutdown.cancel();
        self.strategy_task_tracker.close();
        self.strategy_task_tracker.wait().await;

        info!("Stopping allocation optimizer...");
        self.allocation_shutdown.cancel();
        self.allocation_task_tracker.close();
        self.allocation_task_tracker.wait().await;

        info!("Stopping order manager...");
        self.order_manager_shutdown.cancel();
        self.order_manager_task_tracker.close();
        self.order_manager_task_tracker.wait().await;

        info!("Stopping executor...");
        self.executor_shutdown.cancel();
        self.executor_tracker.close();
        self.executor_tracker.wait().await;

        info!("Stopping persistor...");
        self.persistor_shutdown.cancel();
        self.persistor_task_tracker.close();
        self.persistor_task_tracker.wait().await;
        Ok(())
    }
}
