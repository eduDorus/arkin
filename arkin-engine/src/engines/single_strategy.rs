use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use derive_builder::Builder;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_execution::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;
use arkin_strategies::prelude::*;

use crate::{TradingEngine, TradingEngineError};

#[derive(Builder, Debug)]
pub struct SingleStrategyEngine {
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
    portfolio: Arc<dyn Portfolio>,

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
}

impl SingleStrategyEngine {
    #[instrument(skip_all)]
    async fn load_state(&self) -> Result<(), TradingEngineError> {
        // Setup Insights
        let start = Instant::now();
        let end_time = OffsetDateTime::now_utc();
        // Round to a minute
        let end_time = end_time.replace_second(0).expect("Failed to replace second");
        let end_time = end_time.replace_nanosecond(0).expect("Failed to replace nanosecond");

        let lookback_data = Duration::from_secs(86400);
        let lookback_insights = Duration::from_secs(86400);
        self.insights.load(&self.instruments, end_time, lookback_data).await?;
        let mut clock = Clock::new(end_time - lookback_insights, end_time, Duration::from_secs(60));
        while let Some((_start, end)) = clock.next() {
            self.insights.process(&self.instruments, end).await?;
        }

        // If we are now at the start of a new minute, we need to load the last minute of data
        let diff = OffsetDateTime::now_utc() - end_time;
        if diff > Duration::from_secs(60) {
            info!("Hopping to the next minute to load the last minute of data");
            self.insights
                .load(&self.instruments, end_time + Duration::from_secs(60), Duration::from_secs(60))
                .await?;
            self.insights
                .process(&self.instruments, end_time + Duration::from_secs(60))
                .await?;
        }

        info!("Loaded state in {:?}", start.elapsed());
        Ok(())
    }

    #[instrument(skip_all)]
    async fn pipeline(&self) -> Result<(), TradingEngineError> {
        let mut time_helper = TickHelper::new(Duration::from_secs(60));

        loop {
            tokio::select! {
                (event_time, frequency) = time_helper.tick() => {
                    info!("Processing pipeline for {:?}", event_time);
                    info!("Loading insights...");
                    self.insights.load(&self.instruments, event_time, frequency).await?;

                    info!("Processing insights...");
                    let insights = self.insights.process(&self.instruments, event_time).await?;
                    for insight in &insights {
                        info!("Insight: {}", insight);
                    }

                    info!("Processing strategy...");
                    let signals = self.strategy.insight_update(&self.instruments, event_time, &insights).await?;
                    info!("Adding signals to allocation optimizer...");
                    for signal in &signals {
                        info!("Signal: {}", signal);
                    }
                    self.allocation_optim.new_signals(signals).await?;

                    info!("Processing allocation optimizer...");
                    let execution_orders = self.allocation_optim.optimize(event_time).await?;
                    for order in &execution_orders {
                        info!("Execution Order: {}", order);
                    }

                    info!("Placing orders on the order manager...");
                    self.order_manager.place_orders(execution_orders).await?;
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
    #[instrument(skip_all)]
    async fn start(&self) -> Result<(), TradingEngineError> {
        // Start the persistor
        self.persistor
            .start(self.persistor_task_tracker.clone(), self.persistor_shutdown.clone())
            .await?;

        // Start the portfolio
        self.portfolio
            .start(self.portfolio_task_tracker.clone(), self.portfolio_shutdown.clone())
            .await?;

        // Start the ingestors
        for ingestor in &self.ingestors {
            ingestor
                .start(self.ingestor_task_tracker.clone(), self.ingestor_shutdown.clone())
                .await?;
        }

        // Start the insights
        self.insights
            .start(self.insights_task_tracker.clone(), self.insights_shutdown.clone())
            .await?;

        // Start the strategies
        self.strategy
            .start(self.strategy_task_tracker.clone(), self.strategy_shutdown.clone())
            .await?;

        // Start the allocation optimizer
        self.allocation_optim
            .start(self.allocation_task_tracker.clone(), self.allocation_shutdown.clone())
            .await?;

        // Start the order manager
        self.order_manager
            .start(self.order_manager_task_tracker.clone(), self.order_manager_shutdown.clone())
            .await?;

        // Load the state
        self.load_state().await?;

        // Run the pipeline
        self.pipeline().await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn stop(&self) -> Result<(), TradingEngineError> {
        info!("Stopping ingestors...");
        self.ingestor_shutdown.cancel();
        self.ingestor_task_tracker.close();
        self.ingestor_task_tracker.wait().await;
        for ingestor in &self.ingestors {
            ingestor.cleanup().await?;
        }

        info!("Stopping insights...");
        self.insights_shutdown.cancel();
        self.insights_task_tracker.close();
        self.insights_task_tracker.wait().await;
        self.insights.cleanup().await?;

        info!("Stopping strategies...");
        self.strategy_shutdown.cancel();
        self.strategy_task_tracker.close();
        self.strategy_task_tracker.wait().await;
        self.strategy.cleanup().await?;

        info!("Stopping allocation optimizer...");
        self.allocation_shutdown.cancel();
        self.allocation_task_tracker.close();
        self.allocation_task_tracker.wait().await;
        self.allocation_optim.cleanup().await?;

        info!("Stopping order manager...");
        self.order_manager_shutdown.cancel();
        self.order_manager_task_tracker.close();
        self.order_manager_task_tracker.wait().await;
        self.order_manager.cleanup().await?;

        info!("Stopping persistor...");
        self.persistor_shutdown.cancel();
        self.persistor_task_tracker.close();
        self.persistor_task_tracker.wait().await;
        self.persistor.cleanup().await?;
        Ok(())
    }
}

// impl Engine {
//     pub fn backtest(&self, start: OffsetDateTime, end: OffsetDateTime, frequency_secs: Duration) {
//         let mut clock = Clock::new(start, end, frequency_secs);

//         while let Some(timestamp) = clock.next() {
//             info!("----------------- {:?} -----------------", timestamp);
//             self.run_cycle(timestamp, frequency_secs);
//         }
//         self.portfolio_manager.print_stats();
//         // self.portfolio_manager.print_trades()
//     }

//     fn run_cycle(&self, timestamp: OffsetDateTime, frequency_secs: Duration) {
//         // Snapshot the market and portfolio
//         let market_snapshot = self.market_manager.snapshot(&timestamp, frequency_secs);
//         // for data in market_snapshot.insights() {
//         //     info!("Market data: {}", data);
//         // }
//         let portfolio_snapshot = self.portfolio_manager.snapshot(&timestamp);

//         // Process the insights
//         let insights_snapshot = self.insights_manager.process(&market_snapshot);
//         // for data in insights_snapshot.insights() {
//         //     info!("Insights data: {}", data);
//         // }
//         let strategy_snapshot = self.strategy_manager.process(&insights_snapshot);
//         for signal in &strategy_snapshot.signals {
//             info!("Signal: {}", signal);
//         }

//         let allocation_snapshot =
//             self.allocation_manager
//                 .process(&market_snapshot, &portfolio_snapshot, &strategy_snapshot);
//         for order in &allocation_snapshot.orders {
//             info!("Order: {}", order);
//         }

//         let fills = self.execution_manager.process_backtest(&allocation_snapshot, &market_snapshot);
//         for fill in fills {
//             info!("Fill: {}", fill);
//             self.portfolio_manager.update_position_from_fill(fill.clone());
//         }
//     }
// }
