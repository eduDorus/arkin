use std::sync::Arc;

use async_stream::try_stream;
use futures_util::Stream;
use moka2::future::Cache;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::{debug, error};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::{Instrument, Tick};

use crate::{repos::TickClickhouseRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]

pub struct TickStore {
    instrument_store: Arc<InstrumentStore>,
    tick_repo: TickClickhouseRepo,
    #[builder(default)]
    tick_buffer: Arc<Mutex<Vec<Arc<Tick>>>>,
    #[builder(default = Cache::new(1000))]
    last_tick_cache: Cache<Arc<Instrument>, Arc<Tick>>,
    buffer_size: usize,
}

impl TickStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        // Lock and extract ticks without cloning
        let ticks = {
            let mut lock = self.tick_buffer.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        // Convert to DTOs and insert into the database
        let ticks = ticks.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        debug!("Flushing {} ticks", ticks.len());
        if let Err(e) = self.tick_repo.insert_batch(&ticks).await {
            error!("Failed to flush ticks: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), PersistenceError> {
        let should_commit = {
            let lock = self.tick_buffer.lock().await;
            lock.len() >= self.buffer_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    async fn update_tick_cache(&self, tick: Arc<Tick>) {
        if let Some(cached_tick) = self.last_tick_cache.get(&tick.instrument).await {
            if cached_tick.event_time < tick.event_time {
                self.last_tick_cache.insert(tick.instrument.clone(), tick.clone()).await;
            }
        } else {
            self.last_tick_cache.insert(tick.instrument.clone(), tick.clone()).await;
        }
    }

    pub async fn insert(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        self.update_tick_cache(tick.clone()).await;
        self.tick_repo.insert(tick.into()).await
    }

    pub async fn insert_buffered(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        self.update_tick_cache(tick.clone()).await;
        {
            let mut lock = self.tick_buffer.lock().await;
            lock.push(tick);
        }
        self.commit().await?;
        Ok(())
    }

    pub async fn insert_buffered_vec(&self, ticks: Vec<Arc<Tick>>) -> Result<(), PersistenceError> {
        for tick in &ticks {
            self.update_tick_cache(tick.clone()).await;
        }
        {
            let mut lock = self.tick_buffer.lock().await; // Wait for lock
            lock.extend(ticks);
        }
        self.commit().await?;
        Ok(())
    }

    pub async fn get_last_tick(&self, instrument: &Arc<Instrument>) -> Option<Arc<Tick>> {
        self.last_tick_cache.get(instrument).await
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Arc<Tick>>, PersistenceError> {
        let db_ticks = self.tick_repo.read_range(&instrument_ids, from, to).await?;
        let mut ticks = Vec::with_capacity(db_ticks.len());
        for dto in &db_ticks {
            let instrument = self.instrument_store.read_by_id(&dto.instrument_id).await?;
            let tick = Tick::builder()
                .event_time(dto.event_time)
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
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<impl Stream<Item = Result<Arc<Tick>, PersistenceError>> + '_, PersistenceError> {
        // We do not `async` here, because returning `impl Stream` + `'a` from an `async fn`
        // is not yet stable. Instead, we return a non-async function that constructs the stream.

        // Collect the IDs.
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
                .event_time(row.event_time)
                .instrument(instrument)
                .tick_id(row.tick_id as u64)
                .bid_price(row.bid_price)
                .bid_quantity(row.bid_quantity)
                .ask_price(row.ask_price)
                .ask_quantity(row.ask_quantity)
                .build();

                // Yield the constructed trade to the stream.
                yield Arc::new(tick);
            }
        };
        Ok(stream)
    }
}
