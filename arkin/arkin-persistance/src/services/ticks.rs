use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use crate::repos::TickRepo;

use super::instruments::InstrumentService;

pub struct TickService {
    tick_repo: Arc<TickRepo>,
    instrument_service: Arc<InstrumentService>,
}

impl TickService {
    pub fn new(tick_repo: Arc<TickRepo>, instrument_service: Arc<InstrumentService>) -> Self {
        Self {
            tick_repo,
            instrument_service,
        }
    }

    pub async fn insert(&self, tick: Tick) -> Result<()> {
        self.tick_repo.insert(tick).await
    }

    pub async fn insert_batch(&self, ticks: Vec<Tick>) -> Result<()> {
        self.tick_repo.insert_batch(ticks).await
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Tick>> {
        // Load ticks
        let db_ticks = self.tick_repo.read_range(instrument_ids, from, to).await?;

        let mut ticks = Vec::with_capacity(db_ticks.len());
        for tick in &db_ticks {
            if let Ok(instrument) = self.instrument_service.read_by_id(&tick.instrument_id).await {
                if let Some(instrument) = instrument {
                    ticks.push(Tick {
                        instrument,
                        event_time: tick.event_time,
                        tick_id: tick.tick_id as u64,
                        bid_price: tick.bid_price,
                        bid_quantity: tick.bid_quantity,
                        ask_price: tick.ask_price,
                        ask_quantity: tick.ask_quantity,
                    });
                } else {
                    error!("Instrument not found: {}", tick.instrument_id);
                }
            } else {
                error!("Could not fetch instrument: {}", tick.instrument_id);
            }
        }

        Ok(ticks)
    }
}
