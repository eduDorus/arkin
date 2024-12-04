use std::sync::Arc;

use moka2::future::Cache;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::{Instrument, Tick};

use crate::{repos::TickRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]

pub struct TickStore {
    instrument_store: Arc<InstrumentStore>,
    tick_repo: TickRepo,
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
        if let Err(e) = self.tick_repo.insert_batch(ticks).await {
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
        if let Some(tick) = self.last_tick_cache.get(&tick.instrument).await {
            if tick.event_time < tick.event_time {
                self.last_tick_cache.insert(tick.instrument.clone(), tick.clone()).await;
            }
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
}
