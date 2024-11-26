use std::{collections::HashMap, sync::Arc};

use arkin_core::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn};

use crate::{Portfolio, PortfolioError};

#[derive(Debug, Clone, Builder)]
pub struct SingleStrategyPortfolio {
    pubsub: Arc<PubSub>,
    #[builder(default = "DashMap::new()")]
    positions: DashMap<Arc<Instrument>, Position>,
    #[builder(default = "DashMap::new()")]
    holdings: DashMap<AssetId, Holding>,
}

impl SingleStrategyPortfolio {
    #[instrument(skip_all)]
    fn update_balance(&self, holding: Holding) {
        // Check if we have the asset in the holdings else create
        if self.holdings.contains_key(&holding.asset) {
            self.holdings.alter(&holding.asset, |_, mut h| {
                h.quantity = holding.quantity;
                h
            });
        } else {
            self.holdings.insert(holding.asset.clone(), holding);
        }
    }

    #[instrument(skip_all)]
    fn update_position(&self, fill: ExecutionOrderFill) {
        let position_side = self.positions.get(&fill.instrument).map(|p| p.side);
        if let Some(position_side) = position_side {
            info!("Position side: {:?}", position_side);
            match (position_side, fill.side) {
                (PositionSide::Long, MarketSide::Buy) | (PositionSide::Short, MarketSide::Sell) => {
                    self.increase_position(fill);
                }
                _ => {
                    self.decrease_position(fill);
                }
            }
        } else {
            info!("No position found for instrument: {}", fill.instrument);
            self.create_new_position(fill);
        }
    }

    #[instrument(skip_all)]
    fn increase_position(&self, fill: ExecutionOrderFill) {
        info!("Increasing position: {}", fill.instrument);
        self.positions.alter(&fill.instrument, |_, mut p| {
            let new_total_quantity = p.quantity + fill.quantity;
            p.avg_open_price = (p.avg_open_price * p.quantity + fill.price * fill.quantity) / new_total_quantity;
            p.quantity = new_total_quantity;
            p.commission += fill.commission;
            info!("Increased position: {}", p);
            p
        });
    }

    #[instrument(skip_all)]
    fn decrease_position(&self, fill: ExecutionOrderFill) {
        info!("Decreasing position: {:?}", fill.instrument);
        let initial_quantity = self.positions.get(&fill.instrument).map(|p| p.quantity).unwrap_or_default();

        // Update the position
        self.positions.alter(&fill.instrument, |_, mut p| {
            let close_quantity = fill.quantity.min(p.quantity);
            p.quantity -= close_quantity;
            p.avg_close_price = Some(fill.price);
            p.realized_pnl += match p.side {
                PositionSide::Long => Price::from(fill.price - p.avg_open_price) * close_quantity,
                PositionSide::Short => Price::from(p.avg_open_price - fill.price) * close_quantity,
            };
            p.commission += fill.commission;

            match p.quantity.is_zero() {
                true => {
                    info!("Closed position: {}", p);
                    p.status = PositionStatus::Closed;
                }
                false => {
                    info!("Reduced position: {}", p);
                }
            }
            p
        });

        // If we have remaining quantity, create a new position
        if initial_quantity < fill.quantity {
            let remaining_quantity = fill.quantity - initial_quantity;
            let mut remaining_fill = fill.clone();
            remaining_fill.quantity = remaining_quantity;
            self.create_new_position(fill);
        }
    }

    #[instrument(skip_all)]
    fn create_new_position(&self, fill: ExecutionOrderFill) {
        let position = Position::from(fill);

        info!("Created new position: {}", position);
        self.positions.insert(position.instrument.clone(), position);
    }

    #[instrument(skip_all)]
    fn reconcilliate_position(&self, position: Position) {
        // Check if our position matches the incoming position
        if let Some(mut p) = self.positions.get_mut(&position.instrument) {
            // Update each field and warn if there is a mismatch
            if p.side != position.side {
                warn!("Mismatch in side: {:?} != {:?}", p.side, position.side);
                p.side = position.side;
            }
            if p.avg_open_price != position.avg_open_price {
                warn!(
                    "Mismatch in avg_open_price: {} != {}",
                    p.avg_open_price, position.avg_open_price
                );
                p.avg_open_price = position.avg_open_price;
            }
            if p.avg_close_price != position.avg_close_price {
                warn!(
                    "Mismatch in avg_close_price: {:?} != {:?}",
                    p.avg_close_price, position.avg_close_price
                );
                p.avg_close_price = position.avg_close_price;
            }
            if p.quantity != position.quantity {
                warn!("Mismatch in quantity: {} != {}", p.quantity, position.quantity);
                p.quantity = position.quantity;
            }
            if p.realized_pnl != position.realized_pnl {
                warn!("Mismatch in realized_pnl: {} != {}", p.realized_pnl, position.realized_pnl);
                p.realized_pnl = position.realized_pnl;
            }
            if p.commission != position.commission {
                warn!("Mismatch in commission: {} != {}", p.commission, position.commission);
                p.commission = position.commission;
            }
            if p.status != position.status {
                warn!("Mismatch in status: {:?} != {:?}", p.status, position.status);
                p.status = position.status;
            }
        } else {
            // Create a new position
            info!("Position not found, creating new position: {}", position);
            self.positions.insert(position.instrument.clone(), position);
        }
    }
}

#[async_trait]
impl Portfolio for SingleStrategyPortfolio {
    #[instrument(skip_all)]
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PortfolioError> {
        info!("Starting portfolio...");
        let mut balance_updates = self.pubsub.subscribe::<Holding>();
        let mut position_updates = self.pubsub.subscribe::<Position>();
        let mut fill_updates = self.pubsub.subscribe::<ExecutionOrderFill>();
        loop {
            tokio::select! {
                Ok(balance) = balance_updates.recv() => {
                    if let Err(e) = self.balance_update(balance).await {
                        error!("Failed to process balance update: {}", e);
                    }
                }
                Ok(position) = position_updates.recv() => {
                    if let Err(e) = self.position_update(position).await {
                        error!("Failed to process position update: {}", e);
                    }
                }
                Ok(fill) = fill_updates.recv() => {
                    if let Err(e) = self.fill_update(fill).await {
                        error!("Failed to process fill update: {}", e);
                    }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn position_update(&self, position: Position) -> Result<(), PortfolioError> {
        info!("Processing position update: {}", position);
        self.reconcilliate_position(position);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn balance_update(&self, holding: Holding) -> Result<(), PortfolioError> {
        info!("Processing balance update: {}", holding);
        self.update_balance(holding);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn fill_update(&self, fill: ExecutionOrderFill) -> Result<(), PortfolioError> {
        info!("Processing fill update: {}", fill);
        self.update_position(fill.clone());
        Ok(())
    }

    #[instrument(skip_all)]
    async fn balances(&self) -> HashMap<AssetId, Holding> {
        self.holdings.iter().map(|v| (v.key().clone(), v.value().clone())).collect()
    }

    #[instrument(skip_all)]
    async fn balance(&self, asset: &AssetId) -> Option<Holding> {
        self.holdings.get(asset).map(|v| v.value().clone())
    }

    #[instrument(skip_all)]
    async fn list_positions(&self) -> HashMap<Arc<Instrument>, Position> {
        self.positions.iter().map(|v| (v.key().clone(), v.value().clone())).collect()
    }

    #[instrument(skip_all)]
    async fn list_open_positions_with_quote_asset(&self, quote_asset: &AssetId) -> HashMap<Arc<Instrument>, Position> {
        self.positions
            .iter()
            .filter(|e| e.value().instrument.quote_asset == *quote_asset && e.value().status == PositionStatus::Open)
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    #[instrument(skip_all)]
    async fn capital(&self, asset: &AssetId) -> Notional {
        let current_balance = self.balance(asset).await;
        let current_positions = self.list_open_positions_with_quote_asset(asset).await;
        let positions_value = current_positions.iter().fold(Notional::ZERO, |acc, (_, p)| acc + p.value());

        match current_balance {
            Some(b) => b.quantity + positions_value,
            None => positions_value,
        }
    }

    #[instrument(skip_all)]
    async fn buying_power(&self) -> Notional {
        unimplemented!("buying_power")
    }

    #[instrument(skip_all)]
    async fn total_exposure(&self) -> Notional {
        self.positions.iter().fold(Notional::default(), |acc, e| {
            acc + e.avg_open_price * e.quantity + e.realized_pnl
        })
    }

    #[instrument(skip_all)]
    async fn total_pnl(&self) -> Notional {
        Notional::ZERO
    }

    #[instrument(skip_all)]
    async fn total_commission(&self) -> Notional {
        Notional::ZERO
    }
}

//     pub fn open_position(&self, strategy_id: &StrategyId, instrument: &Instrument) -> Option<Position> {
//         self.positions.get(&(strategy_id.clone(), instrument.clone())).and_then(|v| {
//             if v.value().status == PositionStatus::Open {
//                 Some(v.clone())
//             } else {
//                 None
//             }
//         })
//     }

//     pub fn snapshot(&self, timestamp: OffsetDateTime) -> PortfolioSnapshot {
//         let positions = self.positions.iter().map(|v| v.clone()).collect();
//         PortfolioSnapshot::new(timestamp.to_owned(), self.capital, positions)
//     }

//     pub fn available_capital(&self) -> Notional {
//         self.capital + self.total_pnl() - self.total_commission()
//     }

//     pub fn leverage(&self) -> Decimal {
//         self.leverage
//     }

//     pub fn print_stats(&self) {
//         info!("");
//         info!("------------------ Portfolio Stats --------------------");
//         info!("Capital: {}", self.available_capital().round_dp(2));
//         info!("Return %: {}", self.return_pct().round_dp(5));
//         info!("Profit: {}", self.profit().round_dp(2));
//         info!("Loss: {}", self.loss().round_dp(2));
//         info!("PnL: {}", self.total_pnl().round_dp(2));
//         info!("Commission: {}", self.total_commission().round_dp(2));

//         info!("");
//         info!("Overall Stats:");
//         info!("Trade Volume: {}", self.total_trade_volume().round_dp(2));
//         info!("Trade Count: {}", self.total_trades());
//         info!("Avg Trade Return: {}", self.avg_return_per_trade().round_dp(2));
//         info!("Avg Trade Return Pct: {}", self.avg_return_per_trade_pct().round_dp(5));
//         info!("");
//         info!("Win Trades:");
//         info!("Count: {}", self.total_win_trades());
//         info!("Avg Return: {}", self.avg_return_per_win_trade().round_dp(2));
//         info!("Max Return: {}", self.max_profit_trade().round_dp(2));
//         info!("Avg Return Pct: {}", self.avg_return_per_win_trade_pct().round_dp(5));
//         info!("Max Return Pct: {}", self.max_profit_trade_pct().round_dp(5));
//         info!("");
//         info!("Loss Trades:");
//         info!("Count: {}", self.total_loss_trades());
//         info!("Avg Return: {}", self.avg_loss_per_trade().round_dp(2));
//         info!("Max Return: {}", self.max_loss_trade().round_dp(2));
//         info!("Avg Return Pct: {}", self.avg_loss_per_trade_pct().round_dp(5));
//         info!("Max Return Pct: {}", self.max_loss_trade_pct().round_dp(5));
//         info!("");
//         info!("Advanced Stats:");
//         info!("Win Rate: {}", self.win_rate().round_dp(2));
//         info!("Profit Factor: {}", self.profit_factor().round_dp(2));
//     }

//     pub fn print_trades(&self) {
//         info!("------------------ Portfolio Trades --------------------");
//         for entry in self.position_history.iter() {
//             for position in entry.value() {
//                 info!("- {}", position);
//             }
//         }
//     }

//     pub fn return_pct(&self) -> Decimal {
//         let total_pnl = self.total_pnl();
//         let total_commission = self.total_commission();

//         (self.capital + total_pnl - total_commission) / self.capital
//     }

//     pub fn profit(&self) -> Notional {
//         self.position_history.iter().fold(Notional::default(), |acc, e| {
//             acc + e.value().iter().fold(Notional::default(), |acc, p| {
//                 if p.is_profitable() {
//                     acc + p.realized_pnl
//                 } else {
//                     acc
//                 }
//             })
//         })
//     }

//     pub fn loss(&self) -> Notional {
//         self.position_history.iter().fold(Notional::default(), |acc, e| {
//             acc + e.value().iter().fold(Notional::default(), |acc, p| {
//                 if !p.is_profitable() {
//                     acc + p.realized_pnl
//                 } else {
//                     acc
//                 }
//             })
//         })
//     }

//     pub fn total_pnl(&self) -> Notional {
//         self.position_history.iter().fold(Notional::default(), |acc, e| {
//             acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.realized_pnl)
//         })
//     }

//     pub fn total_commission(&self) -> Notional {
//         self.position_history.iter().fold(Notional::default(), |acc, e| {
//             acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.commission)
//         })
//     }

//     pub fn total_trade_volume(&self) -> Notional {
//         self.position_history.iter().fold(Notional::default(), |acc, e| {
//             acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.trade_volume)
//         })
//     }

//     pub fn total_trades(&self) -> usize {
//         self.position_history.iter().fold(0, |acc, e| acc + e.value().len())
//     }

//     pub fn total_win_trades(&self) -> usize {
//         self.position_history.iter().fold(0, |acc, e| {
//             acc + e.value().iter().fold(0, |acc, p| if p.is_profitable() { acc + 1 } else { acc })
//         })
//     }

//     pub fn total_loss_trades(&self) -> usize {
//         self.position_history.iter().fold(0, |acc, e| {
//             acc + e
//                 .value()
//                 .iter()
//                 .fold(0, |acc, p| if !p.is_profitable() { acc + 1 } else { acc })
//         })
//     }

//     pub fn win_rate(&self) -> Decimal {
//         let win_trades = self.total_win_trades();
//         let trade_count = self.total_trades();
//         if trade_count == 0 {
//             Decimal::ZERO
//         } else {
//             Decimal::from(win_trades) / Decimal::from(trade_count)
//         }
//     }

//     pub fn avg_return_per_trade(&self) -> Notional {
//         let total_pnl = self.total_pnl();
//         let total_trades = self.total_trades();
//         if total_trades == 0 {
//             Notional::default()
//         } else {
//             total_pnl / Decimal::from(total_trades)
//         }
//     }

//     pub fn avg_return_per_trade_pct(&self) -> Decimal {
//         self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
//             acc + e.value().iter().fold(Decimal::ZERO, |acc, p| acc + p.return_pct())
//         }) / Decimal::from(self.total_trades())
//     }

//     pub fn avg_return_per_win_trade(&self) -> Decimal {
//         let profit = self.profit();
//         let total_win_trades = self.total_win_trades();
//         if total_win_trades == 0 {
//             Decimal::ZERO
//         } else {
//             profit / Decimal::from(total_win_trades)
//         }
//     }

//     pub fn avg_return_per_win_trade_pct(&self) -> Decimal {
//         let total_win_trades = self.total_win_trades();
//         if total_win_trades == 0 {
//             Decimal::ZERO
//         } else {
//             self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
//                 acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
//                     if p.is_profitable() {
//                         acc + p.return_pct()
//                     } else {
//                         acc
//                     }
//                 })
//             }) / Decimal::from(total_win_trades)
//         }
//     }

//     pub fn avg_return_per_loss_trade(&self) -> Decimal {
//         let loss = self.loss();
//         let total_loss_trades = self.total_loss_trades();
//         if total_loss_trades == 0 {
//             Decimal::ZERO
//         } else {
//             loss / Decimal::from(total_loss_trades)
//         }
//     }

//     pub fn avg_return_per_loss_trade_pct(&self) -> Decimal {
//         let total_loss_trades = self.total_loss_trades();
//         if total_loss_trades == 0 {
//             Decimal::ZERO
//         } else {
//             self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
//                 acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
//                     if !p.is_profitable() {
//                         acc + p.return_pct()
//                     } else {
//                         acc
//                     }
//                 })
//             }) / Decimal::from(total_loss_trades)
//         }
//     }

//     pub fn max_profit_trade(&self) -> Notional {
//         let mut max_profit = Decimal::ZERO;
//         self.position_history.iter().for_each(|e| {
//             e.value().iter().for_each(|p| {
//                 if p.is_profitable() && p.realized_pnl > max_profit {
//                     max_profit = p.realized_pnl;
//                 }
//             });
//         });
//         max_profit
//     }

//     pub fn max_profit_trade_pct(&self) -> Decimal {
//         let mut max_profit = Decimal::ZERO;
//         self.position_history.iter().for_each(|e| {
//             e.value().iter().for_each(|p| {
//                 if p.is_profitable() && p.return_pct() > max_profit {
//                     max_profit = p.return_pct();
//                 }
//             });
//         });
//         max_profit
//     }

//     pub fn avg_loss_per_trade(&self) -> Decimal {
//         let loss = self.loss();
//         let total_loss_trades = self.total_loss_trades();
//         if total_loss_trades == 0 {
//             Decimal::ZERO
//         } else {
//             loss / Decimal::from(total_loss_trades)
//         }
//     }

//     pub fn avg_loss_per_trade_pct(&self) -> Decimal {
//         let total_loss_trades = self.total_loss_trades();
//         if total_loss_trades == 0 {
//             Decimal::ZERO
//         } else {
//             self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
//                 acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
//                     if !p.is_profitable() {
//                         acc + p.return_pct()
//                     } else {
//                         acc
//                     }
//                 })
//             }) / Decimal::from(total_loss_trades)
//         }
//     }

//     pub fn max_loss_trade(&self) -> Notional {
//         let mut max_loss = Decimal::ZERO;
//         self.position_history.iter().for_each(|e| {
//             e.value().iter().for_each(|p| {
//                 if !p.is_profitable() && p.realized_pnl.abs() > max_loss {
//                     max_loss = p.realized_pnl.abs();
//                 }
//             });
//         });
//         -max_loss
//     }

//     pub fn max_loss_trade_pct(&self) -> Decimal {
//         let mut max_loss = Decimal::ZERO;
//         self.position_history.iter().for_each(|e| {
//             e.value().iter().for_each(|p| {
//                 if !p.is_profitable() && p.return_pct().abs() > max_loss {
//                     max_loss = p.return_pct().abs();
//                 }
//             });
//         });
//         -max_loss
//     }

//     pub fn profit_factor(&self) -> Decimal {
//         let profit = self.profit();
//         let loss = self.loss().abs();
//         if loss == Decimal::ZERO {
//             Decimal::MAX
//         } else {
//             profit / loss
//         }
//     }
// }
