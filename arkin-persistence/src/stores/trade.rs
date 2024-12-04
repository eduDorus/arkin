use std::sync::Arc;

use moka2::future::Cache;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::Trade;

use crate::{repos::TradeRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]

pub struct TradeStore {
    instrument_store: Arc<InstrumentStore>,
    trade_repo: TradeRepo,
    #[builder(default)]
    trade_buffer: Arc<Mutex<Vec<Arc<Trade>>>>,
    #[builder(default = Cache::new(1000))]
    last_trade_cache: Cache<Uuid, Arc<Trade>>,
    buffer_size: usize,
}

impl TradeStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        // Lock and extract trades without cloning
        let trades = {
            let mut lock = self.trade_buffer.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        // Convert to DTOs and insert into the database
        let trades = trades.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if let Err(e) = self.trade_repo.insert_batch(trades).await {
            error!("Failed to flush trades: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), PersistenceError> {
        let should_commit = {
            let lock = self.trade_buffer.lock().await;
            lock.len() >= self.buffer_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    async fn update_trade_cache(&self, tick: Arc<Trade>) {
        if let Some(trade) = self.last_trade_cache.get(&tick.instrument.id).await {
            if trade.event_time < trade.event_time {
                self.last_trade_cache.insert(trade.instrument.id, trade.clone()).await;
            }
        }
    }

    pub async fn insert(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        self.update_trade_cache(trade.clone()).await;
        self.trade_repo.insert(trade.into()).await
    }

    pub async fn insert_buffered(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        self.update_trade_cache(trade.clone()).await;
        {
            let mut lock = self.trade_buffer.lock().await;
            lock.push(trade);
        }
        self.commit().await?;
        Ok(())
    }

    pub async fn insert_buffered_vec(&self, trades: Vec<Arc<Trade>>) -> Result<(), PersistenceError> {
        for trade in &trades {
            self.update_trade_cache(trade.clone()).await;
        }
        {
            let mut lock = self.trade_buffer.lock().await; // Wait for lock
            lock.extend(trades);
        }
        self.commit().await?;
        Ok(())
    }

    pub async fn read_last_trade(&self, instrument_id: &Uuid) -> Option<Arc<Trade>> {
        self.last_trade_cache.get(instrument_id).await
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Arc<Trade>>, PersistenceError> {
        let dto = self.trade_repo.read_range(&instrument_ids, from, to).await?;

        let mut trades = Vec::with_capacity(dto.len());
        for trade in &dto {
            let instrument = self.instrument_store.read_by_id(&trade.instrument_id).await?;
            let trade = Trade::builder()
                .event_time(trade.event_time)
                .instrument(instrument)
                .trade_id(trade.trade_id as u64)
                .side(trade.side)
                .price(trade.price)
                .quantity(trade.quantity)
                .build();
            trades.push(Arc::new(trade));
        }
        Ok(trades)
    }
}
