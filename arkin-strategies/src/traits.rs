use std::sync::Arc;

use async_trait::async_trait;

use arkin_core::prelude::*;

use crate::StrategyError;

#[async_trait]
pub trait StrategyService: RunnableService + Algorithm {}

#[async_trait]
pub trait Algorithm: std::fmt::Debug + Send + Sync {
    async fn insight_tick(&self, tick: Arc<InsightTick>) -> Result<(), StrategyError>;
    // async fn on_tick(&self, tick: Tick) -> Result<(), StrategyError>;
    // async fn on_order(&self, order: Order) -> Result<(), StrategyError>;
    // async fn on_trade(&self, trade: Trade) -> Result<(), StrategyError>;
    // async fn on_event(&self, event: Event) -> Result<(), StrategyError>;
}
