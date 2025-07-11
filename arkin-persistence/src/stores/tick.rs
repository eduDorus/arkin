use std::{collections::HashMap, sync::Arc};

use async_stream::try_stream;
use futures::{stream, Stream, StreamExt};
use time::UtcDateTime;
use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{repos::TickClickhouseRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]
pub struct TickStore {
    instrument_store: Arc<InstrumentStore>,
    tick_repo: Arc<TickClickhouseRepo>,
    #[builder(default)]
    tick_buffer: Arc<Mutex<Vec<Arc<Tick>>>>,
    buffer_size: usize,
    #[builder(default)]
    flush_tracker: TaskTracker,
}

impl TickStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        let mut lock = self.tick_buffer.lock().await;
        let ticks = lock.clone();
        lock.clear();
        drop(lock);

        if ticks.is_empty() {
            debug!("No insights to flush.");
            return Ok(());
        }

        let repo = self.tick_repo.clone();
        let ticks = ticks.into_iter().map(|t| t.into()).collect::<Vec<_>>();

        self.flush_tracker.spawn(async move {
            debug!("Flushing {} ticks", ticks.len());

            // Insert the ticks into the database
            loop {
                match repo.insert_batch(&ticks).await {
                    Ok(_) => {
                        info!("Successfully flushed {} ticks", ticks.len());
                        break;
                    }
                    Err(e) => {
                        error!("Failed to flush ticks: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn insert(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        self.tick_repo.insert(tick.into()).await
    }

    pub async fn insert_buffered(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        let mut lock = self.tick_buffer.lock().await;
        lock.push(tick);

        if lock.len() >= self.buffer_size {
            drop(lock);
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert_buffered_vec(&self, ticks: Vec<Arc<Tick>>) -> Result<(), PersistenceError> {
        let mut lock = self.tick_buffer.lock().await;
        lock.extend(ticks);

        if lock.len() >= self.buffer_size {
            drop(lock);
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: UtcDateTime,
        to: UtcDateTime,
    ) -> Result<Vec<Arc<Tick>>, PersistenceError> {
        let db_ticks = self.tick_repo.read_range(&instrument_ids, from, to).await?;
        let mut ticks = Vec::with_capacity(db_ticks.len());
        for dto in &db_ticks {
            let instrument = self.instrument_store.read_by_id(&dto.instrument_id).await?;
            let tick = Tick::builder()
                .event_time(dto.event_time.to_utc())
                .instrument(instrument)
                .tick_id(dto.tick_id as u64)
                .bid_price(dto.bid_price)
                .bid_quantity(dto.bid_quantity)
                .ask_price(dto.ask_price)
                .ask_quantity(dto.ask_quantity)
                .build();
            ticks.push(Arc::new(tick));
        }
        Ok(ticks)
    }

    pub async fn stream_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: UtcDateTime,
        to: UtcDateTime,
    ) -> Result<impl Stream<Item = Result<Arc<Tick>, PersistenceError>> + '_, PersistenceError> {
        let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
        let mut cursor = self.tick_repo.stream_range(&ids, from, to).await?;

        // Build a "try_stream" that yields trades.
        let stream = try_stream! {
            // Get the async cursor from the repository.

            // Loop over rows in the cursor.
            while let Some(row) = cursor.next().await? {
                // For each row, do your transformations.
                let instrument = self.instrument_store.read_by_id(&row.instrument_id).await?;
                let tick = Tick::builder()
                .event_time(row.event_time.to_utc())
                .instrument(instrument)
                .tick_id(row.tick_id as u64)
                .bid_price(row.bid_price)
                .bid_quantity(row.bid_quantity)
                .ask_price(row.ask_price)
                .ask_quantity(row.ask_quantity)
                .build();
                let tick_arc = Arc::new(tick);
                yield tick_arc;
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
    ) -> impl Stream<Item = Arc<Tick>> + 'static {
        // Split the range into daily chunks
        let time_chunks = datetime_chunks(start, end, frequency).unwrap();
        let instrument_ids = Arc::new(instruments.iter().map(|i| i.id).collect::<Vec<_>>());
        let local_instrument_lookup =
            Arc::new(instruments.iter().map(|i| (i.id, Arc::clone(i))).collect::<HashMap<_, _>>());

        // Clone the repository for use in async closures
        let tick_repo = Arc::clone(&self.tick_repo);

        // Create a stream of futures for each daily chunk
        let fetch_stream = stream::iter(time_chunks).map(move |(start_batch, end_batch)| {
            let tick_repo = Arc::clone(&tick_repo);
            let instrument_ids = instrument_ids.clone();
            let local_instrument_lookup = local_instrument_lookup.clone();

            async move {
                info!("Fetching ticks for batch: {} - {}", start_batch, end_batch);

                // Fetch with retries
                let res = retry(
                    || tick_repo.fetch_batch(&instrument_ids, start_batch, end_batch),
                    5, // Max retries
                )
                .await;

                let batch = res.expect("Failed to fetch batch, abort mission");
                let mut ticks = Vec::with_capacity(batch.len());
                for dto in batch {
                    let instrument = local_instrument_lookup.get(&dto.instrument_id).cloned().unwrap();
                    let tick = Tick::builder()
                        .event_time(dto.event_time.to_utc())
                        .instrument(instrument)
                        .tick_id(dto.tick_id as u64)
                        .bid_price(dto.bid_price)
                        .bid_quantity(dto.bid_quantity)
                        .ask_price(dto.ask_price)
                        .ask_quantity(dto.ask_quantity)
                        .build();
                    ticks.push(Arc::new(tick));
                }
                ticks
            }
        });
        fetch_stream.buffered(buffer_size).flat_map(|x| stream::iter(x))
    }
}
