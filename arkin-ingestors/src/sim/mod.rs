use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures_util::StreamExt;
use time::OffsetDateTime;
use tokio::pin;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::{Ingestor, IngestorError};

#[derive(Debug, TypedBuilder)]
pub struct SimIngestor {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    instruments: Vec<Arc<Instrument>>,
    tick_frequency: Duration,
    start: OffsetDateTime,
    end: OffsetDateTime,
}

#[async_trait]
impl Ingestor for SimIngestor {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), IngestorError> {
        info!("Starting SimIngestor...");

        let mut current_time;
        let mut next_tick_time = self.start + self.tick_frequency;
        let mut rx = self.pubsub.subscribe();

        let tick_stream = self
            .persistence
            .tick_store
            .stream_range(&self.instruments, self.start, self.start)
            .await?;

        let trade_stream = self
            .persistence
            .trade_store
            .stream_range(&self.instruments, self.start, self.end)
            .await?;

        pin!(tick_stream);
        pin!(trade_stream);

        let mut next_tick = tick_stream.next().await;
        let mut next_trade = trade_stream.next().await;

        while next_tick.is_some() || next_trade.is_some() {
            // Convert the `Option<Result<Arc<Tick>, PersistenceError>>` to an Option of the timestamp,
            // to decide which is earlier *by reference only*.
            let tick_ts = match &next_tick {
                Some(Ok(t)) => Some(t.event_time),
                Some(Err(e)) => return Err(IngestorError::PersistenceServiceError(e.to_string())),
                None => None,
            };
            let trade_ts = match &next_trade {
                Some(Ok(t)) => Some(t.event_time),
                Some(Err(e)) => return Err(IngestorError::PersistenceServiceError(e.to_string())),
                None => None,
            };

            // Decide which to replay
            current_time = match (tick_ts, trade_ts) {
                (Some(tick_ts), Some(trade_ts)) => {
                    if tick_ts <= trade_ts {
                        // "take" the tick out of `next_tick`
                        let tick = next_tick.take().unwrap().unwrap();
                        self.pubsub.publish(tick).await;
                        // replace it with the next item from the stream
                        next_tick = tick_stream.next().await;
                        tick_ts
                    } else {
                        let trade = next_trade.take().unwrap().unwrap();
                        self.pubsub.publish(trade).await;
                        next_trade = trade_stream.next().await;
                        trade_ts
                    }
                }
                (Some(ts), None) => {
                    // only ticks left
                    let tick = next_tick.take().unwrap().unwrap();
                    self.pubsub.publish(tick).await;
                    next_tick = tick_stream.next().await;
                    ts
                }
                (None, Some(ts)) => {
                    // only trades left
                    let trade = next_trade.take().unwrap().unwrap();
                    self.pubsub.publish(trade).await;
                    next_trade = trade_stream.next().await;
                    ts
                }
                (None, None) => break, // no more data
            };

            if current_time >= next_tick_time {
                info!("SimIngestor: Publishing IntervalTick at {}", next_tick_time);
                let interval_tick = IntervalTick::builder()
                    .event_time(next_tick_time)
                    .instruments(self.instruments.clone())
                    .frequency(self.tick_frequency)
                    .build();
                self.pubsub.publish(interval_tick).await;
                next_tick_time += self.tick_frequency;

                // Wait for insight tick to be processed
                while let Ok(event) = rx.recv().await {
                    match event {
                        Event::InsightTick(_) => break,
                        _ => {}
                    }
                }
            }
        }

        self.pubsub.publish(Event::Finished).await;

        info!("Simulation ingestor service shutdown...");

        Ok(())
    }
}
