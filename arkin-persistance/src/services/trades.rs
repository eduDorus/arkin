use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use crate::repos::TradeRepo;

use super::instruments::InstrumentService;

#[derive(Debug)]
pub struct TradeService {
    trade_repo: Arc<TradeRepo>,
    instrument_service: Arc<InstrumentService>,
}

impl TradeService {
    pub fn new(trade_repo: Arc<TradeRepo>, instrument_service: Arc<InstrumentService>) -> Self {
        Self {
            trade_repo,
            instrument_service,
        }
    }

    pub async fn insert(&self, trade: Trade) -> Result<()> {
        self.trade_repo.insert(trade).await
    }

    pub async fn insert_batch(&self, trades: Vec<Trade>) -> Result<()> {
        self.trade_repo.insert_batch(trades).await
    }

    pub async fn read_range(
        &self,
        instrument_ids: &[Uuid],
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Trade>> {
        // Load trades
        let db_trades = self.trade_repo.read_range(instrument_ids, from, to).await?;

        let mut trades = Vec::with_capacity(db_trades.len());
        for trade in &db_trades {
            if let Ok(instrument) = self.instrument_service.read_by_id(&trade.instrument_id).await {
                if let Some(instrument) = instrument {
                    trades.push(Trade::new(
                        trade.event_time,
                        instrument,
                        trade.trade_id as u64,
                        MarketSide::from(trade.side.clone()),
                        trade.price,
                        trade.quantity,
                    ));
                } else {
                    error!("Instrument not found: {}", trade.instrument_id);
                }
            } else {
                error!("Could not fetch instrument: {}", trade.instrument_id);
            }
        }

        Ok(trades)
    }
}
