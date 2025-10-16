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
    let _publisher = core_ctx.publisher.clone();
    let shutdown = service_ctx.get_shutdown_token();

    let start = ingestor.start;
    let end = ingestor.end;
    let buffer_size = 3;
    let frequency = if ingestor.end - ingestor.start < Duration::from_secs(86400) {
        Frequency::Hourly
    } else {
        Frequency::Daily
    };

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
            .agg_trade_stream_range_buffered(&instruments, ingestor.start, ingestor.end, buffer_size, frequency)
            .await
            .unwrap(),
        Channel::Ticker => core_ctx
            .persistence
            .tick_stream_range_buffered(&instruments, ingestor.start, ingestor.end, buffer_size, frequency)
            .await
            .unwrap(),
        Channel::FundingRate => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::FundingRate, start, end, buffer_size, frequency)
            .await
            .unwrap(),
        Channel::IndexPriceKlines => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::IndexPrice, start, end, buffer_size, frequency)
            .await
            .unwrap(),
        Channel::MarkPriceKlines => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::MarkPrice, start, end, buffer_size, frequency)
            .await
            .unwrap(),
        Channel::OpenInterest => core_ctx
            .persistence
            .metric_stream_range_buffered(&instruments, MetricType::OpenInterest, start, end, buffer_size, frequency)
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
                frequency,
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
    while let Some(_event) = stream.next().await {
        if shutdown.is_cancelled() {
            info!(target: "ingestor-sim", "shutdown signal received, stopping simulation for venue: {} channel: {}", config.venue, config.channel);
            break;
        }
        debug!(target: "ingestor-sim", "publishing event for venue: {} channel: {}", config.venue, config.channel);
    }
    info!(target: "ingestor-sim", "stream ended for venue: {} channel: {}", config.venue, config.channel);
}

#[async_trait]
impl Runnable for SimIngestor {
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
