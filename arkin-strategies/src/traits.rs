use std::sync::Arc;

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use arkin_core::prelude::*;

use crate::StrategyError;

#[async_trait]
pub trait Algorithm: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), StrategyError>;
    async fn cleanup(&self) -> Result<(), StrategyError>;

    async fn insight_update(
        &self,
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
        insights: &[Insight],
    ) -> Result<Vec<Signal>, StrategyError>;
    // async fn on_tick(&self, tick: Tick) -> Result<(), StrategyError>;
    // async fn on_order(&self, order: Order) -> Result<(), StrategyError>;
    // async fn on_trade(&self, trade: Trade) -> Result<(), StrategyError>;
    // async fn on_event(&self, event: Event) -> Result<(), StrategyError>;
}
