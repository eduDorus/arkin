use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use arkin_core::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::PortfolioError;

#[automock]
#[async_trait]
pub trait Portfolio: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PortfolioError>;

    async fn position_update(&self, position: Position) -> Result<(), PortfolioError>;
    async fn balance_update(&self, holding: Holding) -> Result<(), PortfolioError>;
    async fn fill_update(&self, fill: ExecutionOrderFill) -> Result<(), PortfolioError>;

    async fn balances(&self) -> HashMap<AssetId, Holding>;
    async fn balance(&self, asset: &AssetId) -> Option<Holding>;

    async fn list_positions(&self) -> HashMap<Arc<Instrument>, Position>;
    async fn list_open_positions_with_quote_asset(&self, quote_asset: &AssetId) -> HashMap<Arc<Instrument>, Position>;

    async fn capital(&self, asset: &AssetId) -> Notional;
    async fn buying_power(&self) -> Notional;
    async fn total_exposure(&self) -> Notional;
    async fn total_pnl(&self) -> Notional;
    async fn total_commission(&self) -> Notional;
}
