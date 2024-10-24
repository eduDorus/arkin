use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;

use crate::repos::TickRepo;

use super::instruments::InstrumentService;

#[derive(Debug)]
pub struct TickService {
    tick_repo: TickRepo,
    tick_batch: Mutex<Vec<Tick>>,
    instrument_service: Arc<InstrumentService>,
    batch_size: usize,
}

impl TickService {
    pub fn new(tick_repo: TickRepo, instrument_service: Arc<InstrumentService>, batch_size: usize) -> Self {
        Self {
            tick_repo,
            tick_batch: Mutex::new(Vec::new()),
            instrument_service,
            batch_size,
        }
    }

    pub async fn flush(&self) -> Result<()> {
        // Lock and extract ticks without cloning
        let ticks = {
            let mut lock = self.tick_batch.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        if let Err(e) = self.tick_repo.insert_batch(ticks).await {
            error!("Failed to flush ticks: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<()> {
        let should_commit = {
            let lock = self.tick_batch.lock().await;
            lock.len() >= self.batch_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, tick: Tick) -> Result<()> {
        self.tick_repo.insert(tick).await
    }

    pub async fn insert_batch(&self, tick: Tick) -> Result<()> {
        {
            let mut lock = self.tick_batch.lock().await;
            lock.push(tick);
        }

        self.commit().await?;
        Ok(())
    }

    pub async fn insert_batch_vec(&self, ticks: Vec<Tick>) -> Result<()> {
        {
            let mut lock = self.tick_batch.lock().await; // Wait for lock
            lock.extend(ticks);
        }

        self.commit().await?;
        Ok(())
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Tick>> {
        let instrument_ids = instrument_ids.iter().map(|i| i.id).collect::<Vec<Uuid>>();

        let db_ticks = self.tick_repo.read_range(&instrument_ids, from, to).await?;

        let mut ticks = Vec::with_capacity(db_ticks.len());
        for tick in &db_ticks {
            let instrument = self.instrument_service.read_by_id(tick.instrument_id).await?;
            ticks.push(Tick {
                instrument,
                event_time: tick.event_time,
                tick_id: tick.tick_id as u64,
                bid_price: tick.bid_price,
                bid_quantity: tick.bid_quantity,
                ask_price: tick.ask_price,
                ask_quantity: tick.ask_quantity,
            });
        }

        Ok(ticks)
    }
}
