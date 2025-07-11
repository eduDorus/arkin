use std::{collections::HashMap, sync::Arc};

use async_stream::try_stream;
use futures::{stream, Stream, StreamExt};
use time::UtcDateTime;
use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{repos::TradeClickhouseRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]
pub struct TradeStore {
    instrument_store: Arc<InstrumentStore>,
    trade_repo: Arc<TradeClickhouseRepo>,
    #[builder(default)]
    trade_buffer: Arc<Mutex<Vec<Arc<Trade>>>>,
    buffer_size: usize,
    #[builder(default)]
    flush_tracker: TaskTracker,
}

impl TradeStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        let mut lock = self.trade_buffer.lock().await;
        let trades = lock.clone();
        lock.clear();
        drop(lock);

        if trades.is_empty() {
            debug!("No trades to flush.");
            return Ok(());
        }

        let repo = self.trade_repo.clone();
        let trades = trades.into_iter().map(|t| t.into()).collect::<Vec<_>>();

        self.flush_tracker.spawn(async move {
            debug!("Flushing {} trades", trades.len());

            // Insert the trades into the database
            loop {
                match repo.insert_batch(&trades).await {
                    Ok(_) => {
                        info!("Successfully flushed {} trades", trades.len());
                        break;
                    }
                    Err(e) => {
                        error!("Failed to flush trades: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn insert(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        self.trade_repo.insert(trade.into()).await
    }

    pub async fn insert_buffered(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        let mut lock = self.trade_buffer.lock().await;
        lock.push(trade);

        if lock.len() >= self.buffer_size {
            drop(lock);
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert_buffered_vec(&self, trades: Vec<Arc<Trade>>) -> Result<(), PersistenceError> {
        let mut lock = self.trade_buffer.lock().await;
        lock.extend(trades);

        if lock.len() >= self.buffer_size {
            drop(lock);
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn read_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: UtcDateTime,
        to: UtcDateTime,
    ) -> Result<Vec<Arc<Trade>>, PersistenceError> {
        let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
        let dto = self.trade_repo.read_range(&ids, from, to).await?;

        let mut trades = Vec::with_capacity(dto.len());
        for trade in &dto {
            let instrument = self.instrument_store.read_by_id(&trade.instrument_id).await?;
            let trade = Trade::builder()
                .event_time(trade.event_time.to_utc())
                .instrument(instrument)
                .trade_id(trade.trade_id as u64)
                .side(trade.side.into())
                .price(trade.price)
                .quantity(trade.quantity)
                .build();
            trades.push(Arc::new(trade));
        }
        Ok(trades)
    }

    pub async fn stream_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: UtcDateTime,
        to: UtcDateTime,
    ) -> Result<impl Stream<Item = Result<Arc<Trade>, PersistenceError>> + '_, PersistenceError> {
        // We do not `async` here, because returning `impl Stream` + `'a` from an `async fn`
        // is not yet stable. Instead, we return a non-async function that constructs the stream.

        // Collect the IDs.
        let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
        let mut cursor = self.trade_repo.stream_range(&ids, from, to).await?;

        // Build a "try_stream" that yields trades.
        let stream = try_stream! {
            // Get the async cursor from the repository.

            // Loop over rows in the cursor.
            while let Some(row) = cursor.next().await? {
                // For each row, do your transformations.
                let instrument = self.instrument_store.read_by_id(&row.instrument_id).await?;
                let trade = Trade::builder()
                    .event_time(row.event_time.to_utc())
                    .instrument(instrument)
                    .trade_id(row.trade_id as u64)
                    .side(row.side.into())
                    .price(row.price)
                    .quantity(row.quantity)
                    .build();

                // Yield the constructed trade to the stream.
                yield Arc::new(trade);
            }
        };
        Ok(stream)
    }

    pub async fn stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> impl Stream<Item = Arc<Trade>> + 'static {
        // Split the range into daily chunks
        let time_chunks = datetime_chunks(start, end, frequency).unwrap();
        let instrument_ids = Arc::new(instruments.iter().map(|i| i.id).collect::<Vec<_>>());
        let local_instrument_lookup =
            Arc::new(instruments.iter().map(|i| (i.id, Arc::clone(i))).collect::<HashMap<_, _>>());

        // Clone the repository for use in async closures
        let repo = Arc::clone(&self.trade_repo);

        // Create a stream of futures for each daily chunk
        let fetch_stream = stream::iter(time_chunks).map(move |(start_batch, end_batch)| {
            let repo = Arc::clone(&repo);
            let instrument_ids = instrument_ids.clone();
            let local_instrument_lookup = local_instrument_lookup.clone();

            async move {
                info!("Fetching trades for batch: {} - {}", start_batch, end_batch);

                // Fetch with retries
                let res = retry(
                    || repo.fetch_batch(&instrument_ids, start_batch, end_batch),
                    5, // Max retries
                )
                .await;

                let batch = res.expect("Failed to fetch batch, abort mission");
                let mut trades = Vec::with_capacity(batch.len());
                for dto in batch {
                    let instrument = local_instrument_lookup.get(&dto.instrument_id).cloned().unwrap();
                    let trade = Trade::builder()
                        .event_time(dto.event_time.to_utc())
                        .instrument(instrument)
                        .trade_id(dto.trade_id as u64)
                        .side(dto.side.into())
                        .price(dto.price)
                        .quantity(dto.quantity)
                        .build();
                    trades.push(Arc::new(trade));
                }
                trades
            }
        });
        fetch_stream.buffered(buffer_size).flat_map(|x| stream::iter(x))
    }
}
