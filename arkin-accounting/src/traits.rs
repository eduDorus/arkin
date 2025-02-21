use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;

use arkin_core::prelude::*;

use crate::AccountingError;

#[async_trait]
pub trait AccountingService: RunnableService + Accounting {}

#[async_trait]
pub trait Accounting: std::fmt::Debug + Send + Sync {
    // --- Update Methods ---
    /// Reconciles an external balance update from a venue.
    async fn balance_update(&self, update: Arc<BalanceUpdate>) -> Result<(), AccountingError>;

    /// Reconciles an external position update from a venue.
    async fn position_update(&self, update: Arc<PositionUpdate>) -> Result<(), AccountingError>;

    /// Processes an order fill and updates the ledger.
    async fn order_update(&self, order: Arc<VenueOrder>) -> Result<(), AccountingError>;

    // --- Balance Queries ---
    /// Returns the total balance of an asset on a venue.
    async fn balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal;

    /// Returns the total margin balance of an asset on a venue.
    async fn margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal;

    /// Returns the available margin balance of an asset on a venue for a specific strategy.
    async fn available_margin_balance(&self, venue: &Arc<Venue>, asset: &Arc<Asset>) -> Decimal;

    // --- Position Queries (Global) ---
    /// Returns the current position size for an instrument (e.g., +2 for long, -2 for short).
    async fn position(&self, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns the total open position notional value for an instrument
    async fn position_notional(&self, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns all open positions across instruments.
    async fn positions(&self) -> HashMap<Arc<Instrument>, Decimal>;

    /// Returns all open positions in notional value across instruments.
    async fn positions_notional(&self) -> HashMap<Arc<Instrument>, Decimal>;

    // --- Strategy-Specific Queries ---
    /// Returns the position size for an instrument under a specific strategy.
    async fn strategy_position(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns the total open position notional value for an instrument under a specific strategy.
    async fn strategy_position_notional(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns all open positions for a specific strategy.
    async fn strategy_positions(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal>;

    // Returns all open positions in notional value for a specific strategy.
    async fn strategy_positions_notional(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal>;

    /// Returns the realized PnL for an instrument under a specific strategy.
    async fn strategy_realized_pnl(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns the unrealized PnL for an instrument under a specific strategy.
    async fn strategy_unrealized_pnl(&self, strategy: &Arc<Strategy>, instrument: &Arc<Instrument>) -> Decimal;

    /// Returns the total PnL (realized + unrealized) for a strategy, grouped by asset.
    async fn strategy_total_pnl(&self, strategy: &Arc<Strategy>) -> HashMap<Arc<Asset>, Decimal>;

    // // --- Capital and Buying Power ---
    // /// Returns the total capital (net worth) across all assets.
    // async fn total_capital(&self) -> HashMap<Arc<Asset>, Decimal>;

    // /// Returns the buying power for an asset across all strategies/venues.
    // async fn buying_power(&self, asset: &Arc<Asset>) -> Decimal;

    // /// Returns the buying power for an asset under a specific strategy.
    // async fn strategy_buying_power(&self, strategy: &Arc<Strategy>, asset: &Arc<Asset>) -> Decimal;

    // // --- PnL Queries (Global) ---
    // /// Returns the realized PnL for an instrument across all strategies.
    // async fn realized_pnl(&self, instrument: &Arc<Instrument>) -> Decimal;

    // /// Returns the unrealized PnL for an instrument across all strategies.
    // async fn unrealized_pnl(&self, instrument: &Arc<Instrument>) -> Decimal;

    // /// Returns the total PnL (realized + unrealized) across all instruments, grouped by asset.
    // async fn total_pnl(&self) -> HashMap<Arc<Asset>, Decimal>;

    // // --- Commission Queries ---
    // /// Returns the total commission paid for an instrument across all strategies.
    // async fn commission(&self, instrument: &Arc<Instrument>) -> Decimal;

    // /// Returns the total commission paid across all instruments, grouped by asset.
    // async fn total_commission(&self) -> HashMap<Arc<Asset>, Decimal>;
}
