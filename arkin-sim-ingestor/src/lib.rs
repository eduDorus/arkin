use std::{pin::Pin, sync::Arc, time::Duration};

use arkin_core::prelude::*;
use async_trait::async_trait;
use futures::StreamExt;
use time::UtcDateTime;
use tokio::pin;
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Clone)]
pub struct ReplayTask {
    venue: VenueName,
    channel: Channel,
}

#[derive(TypedBuilder)]
pub struct SimIngestor {
    replay_tasks: Vec<ReplayTask>,
    start: UtcDateTime,
    end: UtcDateTime,
}

async fn replay_task(
    ingestor: Arc<SimIngestor>,
    service_ctx: Arc<ServiceCtx>,
    core_ctx: Arc<CoreCtx>,
    config: ReplayTask,
) {
    info!(target: "ingestor-sim", "starting simulation tasks for venue: {} channel: {}", config.venue, config.channel);
    let shutdown = service_ctx.get_shutdown_token();

    let start = ingestor.start;
    let end = ingestor.end;
    let buffer_size = 3;
    let window = Frequency::Daily;

    let venue = match core_ctx.persistence.get_venue_by_name(&config.venue).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to get venue {}: {}", config.venue, e);
            return;
        }
    };
    let instruments = match core_ctx.persistence.get_instruments_by_venue(&venue).await {
        Ok(ins) => ins,
        Err(e) => {
            error!("Failed to get instruments for venue {}: {}", config.venue, e);
            return;
        }
    };

    let stream = match config.channel {
        Channel::AggTrades => core_ctx
            .persistence
            .agg_trade_stream_range_buffered(&instruments, ingestor.start, ingestor.end, buffer_size, window)
            .await
            .unwrap(),
        Channel::Ticker => core_ctx
            .persistence
            .tick_stream_range_buffered(&instruments, ingestor.start, ingestor.end, buffer_size, window)
            .await
            .unwrap(),
        Channel::FundingRate => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::FundingRate, start, end, buffer_size, window)
            .await
            .unwrap(),
        Channel::IndexPriceKlines => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::IndexPrice, start, end, buffer_size, window)
            .await
            .unwrap(),
        Channel::MarkPriceKlines => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::MarkPrice, start, end, buffer_size, window)
            .await
            .unwrap(),
        Channel::OpenInterest => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::OpenInterest, start, end, buffer_size, window)
            .await
            .unwrap(),
        Channel::LongShortRatio => core_ctx
            .persistence
            .metric_stream_range_buffered(
                &instruments,
                MetricType::CountLongShortRatio,
                start,
                end,
                buffer_size,
                window,
            )
            .await
            .unwrap(),
        _ => {
            error!("Channel {:?} not supported for simulation", config.channel);
            return;
        }
    };

    pin!(stream);

    // Stream into the publisher
    let barrier_window = if let Some(barrier) = core_ctx.simulation_barrier.read().await.as_ref() {
        barrier.window_duration()
    } else {
        Duration::from_secs(3600) // 1 hour windows
    };
    let mut next_barrier_time = start + barrier_window;
    let mut batch = Vec::new();

    while let Some(event) = stream.next().await {
        if shutdown.is_cancelled() {
            info!(target: "ingestor-sim", "shutdown signal received, stopping simulation for venue: {} channel: {}", config.venue, config.channel);
            break;
        }

        // Check if we've reached the barrier boundary
        if next_barrier_time <= event.timestamp() {
            // Publish accumulated batch before waiting at barrier
            if !batch.is_empty() {
                info!(target: "ingestor-sim", "publishing batch of {} events for venue: {} channel: {}", batch.len(), config.venue, config.channel);
                core_ctx.publish_batch(batch.drain(..).collect()).await;
            }

            if let Some(barrier) = core_ctx.simulation_barrier.read().await.as_ref() {
                info!(target: "ingestor-sim", "waiting at barrier for venue: {} channel: {}", config.venue, config.channel);
                barrier.ingestor_confirm_and_wait().await;
                next_barrier_time += barrier_window;
            }
        }

        batch.push(event);
    }

    // Publish any remaining events
    if !batch.is_empty() {
        debug!(target: "ingestor-sim", "publishing final batch of {} events for venue: {} channel: {}", batch.len(), config.venue, config.channel);
        core_ctx.publish_batch(batch).await;
    }

    info!(target: "ingestor-sim", "stream ended for venue: {} channel: {}", config.venue, config.channel);
}

#[async_trait]
impl Runnable for SimIngestor {
    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
        // Count parties
        let parties = self.replay_tasks.len();

        // Determine window duration based on date range (matches the frequency logic in replay_task)
        let window_duration = Duration::from_secs(60);

        let barrier = Arc::new(SyncBarrier::new(parties, window_duration));
        *core_ctx.simulation_barrier.write().await = Some(barrier.clone());

        info!(
            target: "ingestor-sim",
            "Created simulation barrier (window duration: {}s)",
            window_duration.as_secs()
        );
    }

    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        let tasks = self.replay_tasks.clone();

        tasks
            .into_iter()
            .map(|t| replay_task(self.clone(), service_ctx.clone(), core_ctx.clone(), t))
            .map(|f| Box::pin(f) as Pin<Box<dyn Future<Output = ()> + Send>>)
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[tokio::test]
    #[test_log::test]
    async fn test_notify() {
        // We spawn 5 tasks that will print something when notified
        // We will notify them 5 times, with a delay in between
        // if the counter hits 5, we stop notifying

        let barrier = Arc::new(SyncBarrier::new(5, Duration::from_secs(1)));
        let mut counter = 0;

        for i in 0..5 {
            let barrier_clone = barrier.clone();
            tokio::spawn(async move {
                loop {
                    barrier_clone.ingestor_confirm_and_wait().await;
                    info!("Task {} notified", i);
                }
            });
        }
        loop {
            barrier.pubsub_confirm_and_wait().await;
            counter += 1;
            if counter >= 5 {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            barrier.release_ingestors().await;
            info!("Main released ingestors, counter: {}", counter);
        }
    }
}
