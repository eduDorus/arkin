use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;

use arkin_core::prelude::*;

use crate::PortfolioError;

#[async_trait]
pub trait PortfolioService: RunnableService + Accounting {}

#[async_trait]
pub trait Accounting: std::fmt::Debug + Send + Sync {
    /// Update the current balance of a given asset
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn balance_update(&self, update: Arc<BalanceUpdate>) -> Result<(), PortfolioError>;

    /// Update the current position of a given instrument
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn position_update(&self, update: Arc<PositionUpdate>) -> Result<(), PortfolioError>;

    /// Provides the current price of a specific assets in the portfolio
    async fn balance(&self, asset: &Arc<Asset>) -> Option<Arc<BalanceUpdate>>;

    /// Provides the total value of a given asset minus the liabilities. It's the the total net worth in this asset.
    async fn available_balance(&self, asset: &Arc<Asset>) -> Decimal;

    /// Provide the current open position
    async fn get_position_by_instrument(&self, instrument: &Arc<Instrument>) -> Option<Arc<PositionUpdate>>;

    /// Provies a list of all open positions
    async fn get_positions(&self) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>>;

    /// Provides a list of all open positions with a given quote asset
    async fn get_positions_by_quote_asset(&self, asset: &Arc<Asset>) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>>;

    // /// Provides the total value of all assets minus the liabilities. It's the the total net worth in the portfolio.
    // async fn total_capital(&self) -> HashMap<Arc<Asset>, Decimal>;

    // /// The amount of capital available to make new investments or trades.
    // async fn buying_power(&self, asset: &Arc<Asset>) -> Decimal;

    // /// Provies the total buying power available of all assets in the portfolio
    // async fn total_buying_power(&self) -> HashMap<Arc<Asset>, Decimal>;

    // /// Provides the pnl of a given asset
    // async fn pnl_asset(&self, asset: &Arc<Asset>) -> Decimal;

    // /// Provides the pnl of a given instrument
    // async fn pnl_instrument(&self, instrument: &Arc<Instrument>) -> Decimal;

    // /// Provides the total pnl of all assets in the portfolio
    // async fn total_pnl(&self) -> HashMap<Arc<Asset>, Decimal>;

    // /// Provides the commission of a given asset
    // async fn commission_asset(&self, asset: &Arc<Asset>) -> Decimal;

    // /// Provides the commission of a given instrument
    // async fn commission_instrument(&self, instrument: &Arc<Instrument>) -> Decimal;

    // /// Provides the total commission of all assets in the portfolio
    // async fn total_commission(&self) -> HashMap<Arc<Asset>, Decimal>;
}
