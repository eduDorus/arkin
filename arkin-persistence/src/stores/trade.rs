use std::sync::Arc;

use async_stream::try_stream;
use futures_util::Stream;
use moka2::future::Cache;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::{debug, error};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::{Instrument, Trade};

use crate::{repos::TradeClickhouseRepo, PersistenceError};

use super::instrument::InstrumentStore;

#[derive(Debug, Clone, TypedBuilder)]

pub struct TradeStore {
    instrument_store: Arc<InstrumentStore>,
    trade_repo: TradeClickhouseRepo,
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
        debug!("Flushing {} trades", trades.len());
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

    async fn update_trade_cache(&self, trade: Arc<Trade>) {
        if let Some(cached_trade) = self.last_trade_cache.get(&trade.instrument.id).await {
            if cached_trade.event_time < trade.event_time {
                self.last_trade_cache.insert(trade.instrument.id, trade.clone()).await;
            }
        } else {
            self.last_trade_cache.insert(trade.instrument.id, trade.clone()).await;
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
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Arc<Trade>>, PersistenceError> {
        let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
        let dto = self.trade_repo.read_range(&ids, from, to).await?;

        let mut trades = Vec::with_capacity(dto.len());
        for trade in &dto {
            let instrument = self.instrument_store.read_by_id(&trade.instrument_id).await?;
            let trade = Trade::builder()
                .event_time(trade.event_time)
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
        from: OffsetDateTime,
        to: OffsetDateTime,
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
                    .event_time(row.event_time)
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
}
