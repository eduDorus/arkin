use std::{pin::Pin, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use time::UtcDateTime;
use tokio::pin;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

#[derive(TypedBuilder)]
pub struct SimBinanceIngestor {
    instruments: Vec<Arc<Instrument>>,
    #[builder(default = 3)]
    buffer_size: usize,
    start: UtcDateTime,
    end: UtcDateTime,
}

async fn market_replay_task(ingestor: Arc<SimBinanceIngestor>, service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
    info!(target: "ingestor-binance", "starting simulation tasks");

    let publisher = core_ctx.publisher.clone();
    let shutdown = service_ctx.get_shutdown_token();

    let frequency = if ingestor.end - ingestor.start < Duration::from_secs(86400) {
        Frequency::Hourly
    } else {
        Frequency::Daily
    };

    let tick_stream = core_ctx
        .persistence
        .tick_stream_range_buffered(
            &ingestor.instruments,
            ingestor.start,
            ingestor.end,
            ingestor.buffer_size,
            frequency,
        )
        .await;

    let trade_stream = core_ctx
        .persistence
        .trade_stream_range_buffered(
            &ingestor.instruments,
            ingestor.start,
            ingestor.end,
            ingestor.buffer_size,
            frequency,
        )
        .await;

    pin!(tick_stream);
    pin!(trade_stream);

    let mut next_tick = tick_stream.next().await;
    let mut next_trade = trade_stream.next().await;

    while (next_tick.is_some() || next_trade.is_some()) && !shutdown.is_cancelled() {
        // Convert the `Option<Result<Arc<Tick>, PersistenceError>>` to an Option of the timestamp,
        // to decide which is earlier *by reference only*.
        let tick_ts = match &next_tick {
            Some(t) => Some(t.event_time),
            None => None,
        };
        let trade_ts = match &next_trade {
            Some(t) => Some(t.event_time),
            None => None,
        };

        // Decide which to replay
        match (tick_ts, trade_ts) {
            (Some(tick_ts), Some(trade_ts)) => {
                if tick_ts <= trade_ts {
                    let tick = next_tick.take().unwrap();
                    publisher.publish(Event::TickUpdate(tick.into())).await;
                    next_tick = tick_stream.next().await;
                    tick_ts
                } else {
                    let trade = next_trade.take().unwrap();
                    publisher.publish(Event::AggTradeUpdate(trade.into())).await;
                    next_trade = trade_stream.next().await;
                    trade_ts
                }
            }
            (Some(ts), None) => {
                // only ticks left
                let tick = next_tick.take().unwrap();
                publisher.publish(Event::TickUpdate(tick.into())).await;
                next_tick = tick_stream.next().await;
                ts
            }
            (None, Some(ts)) => {
                // only trades left
                let trade = next_trade.take().unwrap();
                publisher.publish(Event::AggTradeUpdate(trade.into())).await;
                next_trade = trade_stream.next().await;
                ts
            }
            (None, None) => break, // no more data
        };
    }
    info!(target: "ingestor-binance", "binance sim ingestor finished task");
}

#[async_trait]
impl Runnable for SimBinanceIngestor {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(market_replay_task(self, service_ctx, core_ctx))]
    }
}
