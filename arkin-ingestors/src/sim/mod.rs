use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use time::UtcDateTime;
use tokio::pin;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

#[derive(TypedBuilder)]
pub struct SimIngestor {
    pubsub: PubSubPublisher,
    persistence: Arc<PersistenceService>,
    instruments: Vec<Arc<Instrument>>,
    tick_frequency: Duration,
    #[builder(default = 3)]
    buffer_size: usize,
    start: UtcDateTime,
    end: UtcDateTime,
}

#[async_trait]
impl RunnableService for SimIngestor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting SimIngestor...");

        let mut current_time;
        let mut next_tick_time = self.start + self.tick_frequency;

        let tick_stream = self
            .persistence
            .tick_store
            .stream_range_buffered(&self.instruments, self.start, self.end, self.buffer_size, Frequency::Daily)
            .await;

        let trade_stream = self
            .persistence
            .trade_store
            .stream_range_buffered(&self.instruments, self.start, self.end, self.buffer_size, Frequency::Daily)
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
            current_time = match (tick_ts, trade_ts) {
                (Some(tick_ts), Some(trade_ts)) => {
                    if tick_ts <= trade_ts {
                        // "take" the tick out of `next_tick`
                        let tick = next_tick.take().unwrap();
                        self.pubsub.publish(tick).await;
                        // replace it with the next item from the stream
                        next_tick = tick_stream.next().await;
                        tick_ts
                    } else {
                        let trade = next_trade.take().unwrap();
                        self.pubsub.publish(trade).await;
                        next_trade = trade_stream.next().await;
                        trade_ts
                    }
                }
                (Some(ts), None) => {
                    // only ticks left
                    let tick = next_tick.take().unwrap();
                    self.pubsub.publish(tick).await;
                    next_tick = tick_stream.next().await;
                    ts
                }
                (None, Some(ts)) => {
                    // only trades left
                    let trade = next_trade.take().unwrap();
                    self.pubsub.publish(trade).await;
                    next_trade = trade_stream.next().await;
                    ts
                }
                (None, None) => break, // no more data
            };

            if current_time > next_tick_time {
                // info!("SimIngestor: Publishing IntervalTick at {}", next_tick_time);
                let interval_tick = InsightsTick::builder()
                    .event_time(next_tick_time)
                    .instruments(self.instruments.clone())
                    .frequency(self.tick_frequency)
                    .build();
                self.pubsub.publish(interval_tick).await;
                next_tick_time += self.tick_frequency;
            }
        }

        // self.pubsub.publish(Event::Finished).await;
        info!("Simulation ingestor service stopped.");
        Ok(())
    }
}
