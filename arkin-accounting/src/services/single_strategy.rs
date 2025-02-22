use std::{collections::HashMap, sync::Arc};

use arkin_core::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::Decimal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use crate::{Accounting, AccountingError, AccountingService};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SingleStrategyPortfolio {
    pubsub: Arc<PubSub>,
    #[builder(default = DashMap::new())]
    _positions: DashMap<Arc<Instrument>, Arc<PositionUpdate>>,
    #[builder(default = DashMap::new())]
    balances: DashMap<Arc<Asset>, Arc<BalanceUpdate>>,
}

impl SingleStrategyPortfolio {
    pub fn add_balance(&self, balance: Arc<BalanceUpdate>) {
        self.balances.insert(balance.asset.clone(), balance);
    }
}

#[async_trait]
impl Accounting for SingleStrategyPortfolio {
    // async fn balance_update(&self, update: Arc<BalanceUpdate>) -> Result<(), AccountingError> {
    //     info!("Portfolio processing balance update: {}", update);
    //     self.balances.insert(update.asset.clone(), update);
    //     Ok(())
    // }

    // async fn position_update(&self, update: Arc<PositionUpdate>) -> Result<(), AccountingError> {
    //     info!("Portfolio processing position update: {}", update);
    //     self.positions.insert(update.instrument.clone(), update);
    //     Ok(())
    // }

    // async fn balance(&self, asset: &Arc<Asset>) -> Option<Arc<BalanceUpdate>> {
    //     self.balances.get(asset).map(|v| v.value().clone())
    // }

    // async fn available_balance(&self, asset: &Arc<Asset>) -> Decimal {
    //     let current_balance = self.balance(asset).await;
    //     let current_positions = self.get_positions_by_quote_asset(asset).await;
    //     let positions_value = current_positions
    //         .iter()
    //         .fold(Decimal::ZERO, |acc, (_, s)| acc + s.notional_value());

    //     match current_balance {
    //         Some(b) => b.quantity + positions_value,
    //         None => positions_value,
    //     }
    // }

    // async fn get_position_by_instrument(&self, instrument: &Arc<Instrument>) -> Option<Arc<PositionUpdate>> {
    //     self.positions.get(instrument).map(|v| v.value().clone())
    // }

    // async fn get_positions(&self) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
    //     self.positions.iter().map(|e| (e.key().clone(), e.value().clone())).collect()
    // }

    // async fn get_positions_by_quote_asset(
    //     &self,
    //     quote_asset: &Arc<Asset>,
    // ) -> HashMap<Arc<Instrument>, Arc<PositionUpdate>> {
    //     self.positions
    //         .iter()
    //         .filter_map(|e| {
    //             if e.key().quote_asset == *quote_asset {
    //                 Some((e.key().clone(), e.value().clone()))
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect()
    // }
    // --- Update Methods ---
    // --- Update Methods ---
    /// Reconciles an external balance update from a venue.
    async fn balance_update(&self, _update: Arc<BalanceUpdate>) -> Result<(), AccountingError> {
        todo!()
    }

    /// Reconciles an external position update from a venue.
    async fn position_update(&self, _update: Arc<PositionUpdate>) -> Result<(), AccountingError> {
        todo!()
    }

    /// Processes an order fill and updates the ledger.
    async fn order_update(&self, _order: Arc<VenueOrder>) -> Result<(), AccountingError> {
        todo!()
    }

    // --- Balance Queries ---
    /// Return the total balance of an all assets on a venue
    async fn balance(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        todo!()
    }

    /// Returns the total margin balance of all assets on a venue.
    async fn margin_balance(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        todo!()
    }

    async fn available_margin_balance(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Asset>, Decimal> {
        todo!()
    }

    /// Returns the total balance of an asset on a venue.
    async fn asset_balance(&self, _venue: &Arc<Venue>, _asset: &Arc<Asset>) -> Decimal {
        todo!()
    }

    /// Returns the total margin balance of an asset on a venue.
    async fn asset_margin_balance(&self, _venue: &Arc<Venue>, _asset: &Arc<Asset>) -> Decimal {
        todo!()
    }

    /// Returns the available margin balance of an asset on a venue for a specific strategy.
    async fn asset_available_margin_balance(&self, _venue: &Arc<Venue>, _asset: &Arc<Asset>) -> Decimal {
        todo!()
    }

    // --- Position Queries (Global) ---
    /// Returns all open positions across instruments.
    async fn position(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
    }

    /// Returns all open positions in notional value across instruments.
    async fn position_value(&self, _venue: &Arc<Venue>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
    }

    /// Returns the current position size for an instrument (e.g., +2 for long, -2 for short).
    async fn instrument_position(&self, _instrument: &Arc<Instrument>) -> Decimal {
        todo!()
    }

    /// Returns the total open position notional value for an instrument
    // async fn instrument_value(&self, _instrument: &Arc<Instrument>) -> Decimal {
    //     todo!()
    // }

    // --- Strategy-Specific Queries ---
    /// Returns the position size for an instrument under a specific strategy.
    async fn strategy_instrument_position(&self, _strategy: &Arc<Strategy>, _instrument: &Arc<Instrument>) -> Decimal {
        todo!()
    }

    /// Returns the total open position notional value for an instrument under a specific strategy.
    async fn strategy_instrument_position_value(
        &self,
        _strategy: &Arc<Strategy>,
        _instrument: &Arc<Instrument>,
    ) -> Decimal {
        todo!()
    }

    /// Returns all open positions for a specific strategy.
    async fn strategy_position(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
    }

    // Returns all open positions in notional value for a specific strategy.
    async fn strategy_position_value(&self, _strategy: &Arc<Strategy>) -> HashMap<Arc<Instrument>, Decimal> {
        todo!()
    }

    /// Returns the realized PnL for an instrument under a specific strategy.
    async fn strategy_realized_pnl(&self, _strategy: &Arc<Strategy>) -> Decimal {
        todo!()
    }
}

#[async_trait]
impl RunnableService for SingleStrategyPortfolio {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting portfolio...");

        let mut rx = self.pubsub.subscribe();

        loop {
            tokio::select! {
                Ok(event) = rx.recv() => {
                    match event {
                        Event::BalanceUpdate(update) => {
                            if let Err(e) = self.balance_update(update).await {
                                error!("Failed to process balance update: {}", e);
                            }
                        }
                        Event::PositionUpdate(update) => {
                            if let Err(e) = self.position_update(update).await {
                                error!("Failed to process position update: {}", e);
                            }
                        }
                        _ => {}
                      }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AccountingService for SingleStrategyPortfolio {}
