use std::sync::Arc;

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
    start: OffsetDateTime,
    end: OffsetDateTime,
}

#[async_trait]
impl Ingestor for SimIngestor {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), IngestorError> {
        info!("Starting SimIngestor...");

        let tick_stream = self
            .persistence
            .tick_store
            .stream_range(&self.instruments, self.start, self.end)
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
            match (tick_ts, trade_ts) {
                (Some(tick_time), Some(trade_time)) => {
                    if tick_time <= trade_time {
                        // "take" the tick out of `next_tick`
                        let tick = next_tick.take().unwrap().unwrap();
                        self.pubsub.publish(tick).await;
                        // replace it with the next item from the stream
                        next_tick = tick_stream.next().await;
                    } else {
                        let trade = next_trade.take().unwrap().unwrap();
                        self.pubsub.publish(trade).await;
                        next_trade = trade_stream.next().await;
                    }
                }
                (Some(_), None) => {
                    // only ticks left
                    let tick = next_tick.take().unwrap().unwrap();
                    self.pubsub.publish(tick).await;
                    next_tick = tick_stream.next().await;
                }
                (None, Some(_)) => {
                    // only trades left
                    let trade = next_trade.take().unwrap().unwrap();
                    self.pubsub.publish(trade).await;
                    next_trade = trade_stream.next().await;
                }
                (None, None) => break, // no more data
            }
        }

        self.pubsub.publish(Event::Finished).await;

        Ok(())
    }
}
