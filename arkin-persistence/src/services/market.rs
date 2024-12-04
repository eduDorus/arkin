use std::sync::Arc;

use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{
    stores::{AssetStore, InstrumentStore, TickStore, TradeStore},
    PersistenceError,
};

#[derive(Debug, Clone, TypedBuilder)]

pub struct MarketService {
    pub asset_store: AssetStore,
    pub instrument_store: InstrumentStore,
    pub tick_store: TickStore,
    pub trade_store: TradeStore,
}

impl MarketService {
    pub async fn insert_asset(&self, asset: Arc<Asset>) -> Result<(), PersistenceError> {
        self.asset_store.insert(asset).await
    }

    pub async fn read_asset_by_id(&self, id: &Uuid) -> Result<Arc<Asset>, PersistenceError> {
        self.asset_store.read_by_id(id).await
    }

    pub async fn read_asset_by_symbol(&self, name: &str) -> Result<Arc<Asset>, PersistenceError> {
        self.asset_store.read_by_symbol(name).await
    }

    pub async fn insert_instrument(&self, instrument: Arc<Instrument>) -> Result<(), PersistenceError> {
        self.instrument_store.insert(instrument).await
    }

    pub async fn read_instrument_by_id(&self, id: &Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        self.instrument_store.read_by_id(id).await
    }

    pub async fn read_instrument_by_venue_symbol(&self, symbol: &str) -> Result<Arc<Instrument>, PersistenceError> {
        self.instrument_store.read_by_venue_symbol(symbol).await
    }

    pub async fn insert_tick(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        self.tick_store.insert(tick).await
    }

    pub async fn insert_tick_buffered(&self, tick: Arc<Tick>) -> Result<(), PersistenceError> {
        self.tick_store.insert_buffered(tick).await
    }

    pub async fn insert_tick_buffered_vec(&self, ticks: Vec<Arc<Tick>>) -> Result<(), PersistenceError> {
        self.tick_store.insert_buffered_vec(ticks).await
    }

    pub async fn read_ticks_range(
        &self,
        instrument_id: &[Uuid],
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Arc<Tick>>, PersistenceError> {
        self.tick_store.read_range(instrument_id, start, end).await
    }

    pub async fn insert_trade(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        self.trade_store.insert(trade).await
    }

    pub async fn insert_trade_buffered(&self, trade: Arc<Trade>) -> Result<(), PersistenceError> {
        self.trade_store.insert_buffered(trade).await
    }

    pub async fn insert_trade_buffered_vec(&self, trades: Vec<Arc<Trade>>) -> Result<(), PersistenceError> {
        self.trade_store.insert_buffered_vec(trades).await?;
        Ok(())
    }

    pub async fn read_trades_range(
        &self,
        instruments: &[Arc<Instrument>],
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Arc<Trade>>, PersistenceError> {
        self.trade_store.read_range(instruments, start, end).await
    }
}
