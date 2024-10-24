use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tracing::error;

use crate::repos::TradeRepo;

use super::instruments::InstrumentService;

#[derive(Debug)]
pub struct TradeService {
    trade_repo: Arc<TradeRepo>,
    trade_batch: Arc<Mutex<Vec<Trade>>>,
    instrument_service: Arc<InstrumentService>,
    batch_size: usize,
}

impl TradeService {
    pub fn new(trade_repo: Arc<TradeRepo>, instrument_service: Arc<InstrumentService>, batch_size: usize) -> Self {
        Self {
            trade_repo,
            trade_batch: Arc::new(Mutex::new(Vec::new())),
            instrument_service,
            batch_size,
        }
    }

    pub async fn flush(&self) -> Result<()> {
        // Lock and extract trades without cloning
        let trades = {
            let mut lock = self.trade_batch.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        if let Err(e) = self.trade_repo.insert_batch(trades).await {
            error!("Failed to flush ticks: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<()> {
        let should_commit = {
            let lock = self.trade_batch.lock().await;
            lock.len() >= self.batch_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, trade: Trade) -> Result<()> {
        self.trade_repo.insert(trade).await
    }

    pub async fn insert_batch(&self, trade: Trade) -> Result<()> {
        {
            let mut lock = self.trade_batch.lock().await;
            lock.push(trade);
        }

        self.commit().await?;
        Ok(())
    }

    pub async fn insert_batch_vec(&self, trades: Vec<Trade>) -> Result<()> {
        {
            let mut lock = self.trade_batch.lock().await; // Wait for lock
            lock.extend(trades);
        }

        self.commit().await?;
        Ok(())
    }

    pub async fn read_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Trade>> {
        let instrument_ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();

        let db_trades = self.trade_repo.read_range(&instrument_ids, from, to).await?;

        let mut trades = Vec::with_capacity(db_trades.len());
        for trade in &db_trades {
            let instrument = self.instrument_service.read_by_id(trade.instrument_id).await?;
            trades.push(Trade::new(
                trade.event_time,
                instrument,
                trade.trade_id as u64,
                MarketSide::from(trade.side.clone()),
                trade.price,
                trade.quantity,
            ));
        }

        Ok(trades)
    }
}
