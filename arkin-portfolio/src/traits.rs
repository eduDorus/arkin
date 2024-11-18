use std::sync::Arc;

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use arkin_core::prelude::*;

use crate::PortfolioError;

#[async_trait]
pub trait Portfolio: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), PortfolioError>;
    async fn cleanup(&self) -> Result<(), PortfolioError>;
    async fn update_position(
        &self,
        event_time: OffsetDateTime,
        symbol: Arc<Instrument>,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
    ) -> Result<(), PortfolioError>;
    async fn positions(&self) -> Vec<Position>;
    async fn capital(&self) -> Notional;
    async fn buying_power(&self) -> Notional;
    async fn total_exposure(&self) -> Notional;
    async fn total_pnl(&self) -> Notional;
    async fn total_commission(&self) -> Notional;
}
