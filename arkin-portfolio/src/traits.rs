use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use rust_decimal::prelude::*;
use tokio_util::sync::CancellationToken;

use arkin_core::prelude::*;

use crate::PortfolioError;

#[automock]
#[async_trait]
pub trait Accounting: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PortfolioError>;

    /// Update the current price of a given instrument
    async fn price_update(&self, tick: Arc<Tick>) -> Result<(), PortfolioError>;

    /// Update the current balance of a given asset
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn balance_update(&self, holding: Arc<Holding>) -> Result<(), PortfolioError>;

    /// Update the current position of a given instrument
    /// This comes from the exchange and should be reconciled with the portfolio
    async fn position_update(&self, position: Arc<Position>) -> Result<(), PortfolioError>;

    /// Update the current fill of a given order
    /// This comes from the order manager and updates a given position
    async fn fill_update(&self, fill: Arc<VenueOrderFill>) -> Result<(), PortfolioError>;

    /// Provides the current price of a specific assets in the portfolio
    async fn balance(&self, asset: &Arc<Asset>) -> Option<Holding>;

    /// Provides the current price of all assets in the portfolio
    async fn total_balance(&self) -> HashMap<Arc<Asset>, Arc<Holding>>;

    /// Provides a list off all positions with a given quote asset
    async fn list_positions_with_quote_asset(
        &self,
        asset: &Arc<Asset>,
    ) -> HashMap<Arc<Instrument>, BTreeSet<Arc<Position>>>;

    /// Provides a list of all positions with a given instrument
    async fn list_positions_with_instrument(
        &self,
        instrument: &Arc<Instrument>,
    ) -> HashMap<Arc<Instrument>, BTreeSet<Arc<Position>>>;

    /// Provide the current open position
    async fn get_open_position(&self, instrument: &Arc<Instrument>) -> Option<Arc<Position>>;

    /// Provies a list of all open positions
    async fn list_open_positions(&self) -> HashMap<Arc<Instrument>, Arc<Position>>;

    /// Provides a list of all open positions with a given quote asset
    async fn list_open_positions_with_quote_asset(&self, asset: &Arc<Asset>)
        -> HashMap<Arc<Instrument>, Arc<Position>>;

    /// Provides a list of all closed positions
    async fn list_closed_positions(&self) -> HashMap<Arc<Instrument>, BTreeSet<Arc<Position>>>;

    /// Provides the total value of a given asset minus the liabilities. It's the the total net worth in this asset.
    async fn capital(&self, asset: &Arc<Asset>) -> Decimal;

    /// Provides the total value of all assets minus the liabilities. It's the the total net worth in the portfolio.
    async fn total_capital(&self) -> HashMap<Arc<Asset>, Decimal>;

    /// The amount of capital available to make new investments or trades.
    async fn buying_power(&self, asset: &Arc<Asset>) -> Decimal;

    /// Provies the total buying power available of all assets in the portfolio
    async fn total_buying_power(&self) -> HashMap<Arc<Asset>, Decimal>;

    /// Provides the pnl of a given asset
    async fn pnl_asset(&self, asset: &Arc<Asset>) -> Decimal;

    /// Provides the pnl of a given instrument
    async fn pnl_instrument(&self, instrument: &Arc<Instrument>) -> Decimal;

    /// Provides the total pnl of all assets in the portfolio
    async fn total_pnl(&self) -> HashMap<Arc<Asset>, Decimal>;

    /// Provides the commission of a given asset
    async fn commission_asset(&self, asset: &Arc<Asset>) -> Decimal;

    /// Provides the commission of a given instrument
    async fn commission_instrument(&self, instrument: &Arc<Instrument>) -> Decimal;

    /// Provides the total commission of all assets in the portfolio
    async fn total_commission(&self) -> HashMap<Arc<Asset>, Decimal>;
}
