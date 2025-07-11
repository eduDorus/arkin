use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use time::UtcDateTime;
use tokio::pin;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

#[derive(TypedBuilder)]
pub struct SimBinanceIngestor {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    persistence: Arc<Persistence>,
    instruments: Vec<Arc<Instrument>>,
    #[builder(default = 3)]
    buffer_size: usize,
    start: UtcDateTime,
    end: UtcDateTime,
}

#[async_trait]
impl Runnable for SimBinanceIngestor {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    async fn start_tasks(self: Arc<Self>, ctx: Arc<ServiceCtx>) {
        info!(target: "ingestor-binance", "starting simulation tasks");

        let publisher = self.publisher.clone();
        let shutdown = ctx.get_shutdown_token();

        let frequency = if self.end - self.start < Duration::from_secs(86400) {
            Frequency::Hourly
        } else {
            Frequency::Daily
        };

        let tick_stream = self
            .persistence
            .tick_store
            .stream_range_buffered(&self.instruments, self.start, self.end, self.buffer_size, frequency)
            .await;

        let trade_stream = self
            .persistence
            .trade_store
            .stream_range_buffered(&self.instruments, self.start, self.end, self.buffer_size, frequency)
            .await;

        ctx.spawn(async move {
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
                            publisher.publish(Event::TradeUpdate(trade.into())).await;
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
                        publisher.publish(Event::TradeUpdate(trade.into())).await;
                        next_trade = trade_stream.next().await;
                        ts
                    }
                    (None, None) => break, // no more data
                };
            }
            info!(target: "ingestor-binance", "binance sim ingestor finished task");
        });
    }
}
