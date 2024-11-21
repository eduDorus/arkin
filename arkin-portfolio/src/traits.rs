use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use arkin_core::prelude::*;

use crate::PortfolioError;

#[automock]
#[async_trait]
pub trait Portfolio: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), PortfolioError>;
    async fn cleanup(&self) -> Result<(), PortfolioError>;

    // Update
    async fn position_update(&self, position: Position) -> Result<(), PortfolioError>;
    async fn fill_update(&self, fill: Fill) -> Result<(), PortfolioError>;
    async fn balance_update(&self, holding: Holding) -> Result<(), PortfolioError>;

    async fn balances(&self) -> HashMap<String, Holding>;
    async fn positions(&self) -> HashMap<Arc<Instrument>, Position>;
    async fn capital(&self) -> Notional;
    async fn buying_power(&self) -> Notional;
    async fn total_exposure(&self) -> Notional;
    async fn total_pnl(&self) -> Notional;
    async fn total_commission(&self) -> Notional;
}
