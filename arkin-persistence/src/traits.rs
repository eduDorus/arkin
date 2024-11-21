use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::PersistenceError;

#[automock]
#[async_trait]
pub trait Persistor: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), PersistenceError>;
    async fn cleanup(&self) -> Result<(), PersistenceError>;
    async fn flush(&self) -> Result<(), PersistenceError>;

    async fn insert_instrument(&self, instrument: Instrument) -> Result<(), PersistenceError>;
    async fn read_instrument_by_id(&self, id: Uuid) -> Result<Arc<Instrument>, PersistenceError>;
    async fn read_instrument_by_venue_symbol(&self, venue_symbol: String) -> Result<Arc<Instrument>, PersistenceError>;

    async fn insert_tick(&self, tick: Tick) -> Result<(), PersistenceError>;
    async fn insert_tick_batch(&self, tick: Tick) -> Result<(), PersistenceError>;
    async fn insert_tick_batch_vec(&self, ticks: Vec<Tick>) -> Result<(), PersistenceError>;
    async fn read_latest_tick(
        &self,
        event_time: OffsetDateTime,
        instrument: &Arc<Instrument>,
    ) -> Result<Option<Tick>, PersistenceError>;

    async fn read_trades_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Trade>, PersistenceError>;

    async fn insert_trade(&self, trade: Trade) -> Result<(), PersistenceError>;
    async fn insert_trade_batch(&self, trade: Trade) -> Result<(), PersistenceError>;
    async fn insert_trade_batch_vec(&self, trades: Vec<Trade>) -> Result<(), PersistenceError>;

    async fn read_ticks_range(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<Vec<Tick>, PersistenceError>;
    async fn insert_insight(&self, insight: Insight) -> Result<(), PersistenceError>;
    async fn insert_insight_batch(&self, insight: Insight) -> Result<(), PersistenceError>;
    async fn insert_insight_batch_vec(&self, insights: Vec<Insight>) -> Result<(), PersistenceError>;
}
