use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument, warn};

use crate::{Portfolio, PortfolioError};

#[derive(Debug, Clone, Builder)]
pub struct SingleStrategyPortfolio {
    capital: Notional,
    #[builder(default = "Decimal::ONE")]
    leverage: Decimal,
    #[builder(default = "DashMap::new()")]
    positions: DashMap<Arc<Instrument>, Position>,
    #[builder(default = "DashMap::new()")]
    balances: DashMap<String, Holding>,
}

impl SingleStrategyPortfolio {
    fn update_balance(&self, holding: Holding) {
        if let Some(mut balance) = self.balances.get_mut(&holding.asset) {
            balance.quantity = holding.quantity;
        } else {
            self.balances.insert(holding.asset.clone(), holding);
        }
    }
    fn update_position(&self, fill: Fill) {
        if let Some(mut position) = self.positions.get_mut(&fill.instrument) {
            let (decreasing, increasing) = match (position.side, fill.side) {
                (PositionSide::Long, MarketSide::Sell) | (PositionSide::Short, MarketSide::Buy) => (true, false),
                _ => (false, true),
            };

            if increasing {
                self.increase_position(&mut position, fill);
            } else if decreasing {
                let remaining = self.decrease_position(&mut position, fill.clone());
                // Close position
                if position.quantity.is_zero() {
                    self.positions.remove(&position.instrument);
                }

                // If there is some remaining quantity, create a new position
                if let Some(remaining_quantity) = remaining {
                    let mut remaining_fill = fill;
                    remaining_fill.quantity = remaining_quantity;
                    self.create_new_position(remaining_fill);
                }
            }
        } else {
            self.create_new_position(fill);
        }
    }

    fn increase_position(&self, position: &mut Position, fill: Fill) {
        let new_total_quantity = position.quantity + fill.quantity;
        position.avg_open_price =
            (position.avg_open_price * position.quantity + fill.price * fill.quantity) / new_total_quantity;
        position.quantity = new_total_quantity;
        position.commission += fill.commission;
        position.updated_at = fill.created_at;

        info!("Increased position: {}", position);
    }

    fn decrease_position(&self, position: &mut Position, fill: Fill) -> Option<Quantity> {
        let close_quantity = position.quantity.min(fill.quantity);
        let pnl = self.calculate_pnl(position, fill.price, close_quantity);

        position.realized_pnl += pnl;
        position.quantity -= close_quantity;
        position.avg_close_price = Some(fill.price);
        position.commission += fill.commission;
        position.updated_at = fill.created_at;

        if position.quantity.is_zero() {
            position.status = PositionStatus::Closed;
        }
        info!("Reduced position: {}", position);

        if close_quantity < fill.quantity {
            Some(fill.quantity - close_quantity)
        } else {
            None
        }
    }

    fn create_new_position(&self, fill: Fill) {
        let position = Position::from(fill);

        info!("Created new position: {}", position);
        self.positions.insert(position.instrument.clone(), position);
    }

    fn calculate_pnl(&self, position: &Position, close_price: Price, close_quantity: Quantity) -> Notional {
        match position.side {
            PositionSide::Long => Price::from(close_price - position.avg_open_price) * close_quantity,
            PositionSide::Short => Price::from(position.avg_open_price - close_price) * close_quantity,
        }
    }

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
            warn!("Position not found, creating new position: {}", position);
            self.positions.insert(position.instrument.clone(), position);
        }
    }
}

#[async_trait]
impl Portfolio for SingleStrategyPortfolio {
    #[instrument(skip(self))]
    async fn start(&self, _task_tracker: TaskTracker, _shutdown: CancellationToken) -> Result<(), PortfolioError> {
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), PortfolioError> {
        Ok(())
    }

    #[instrument(skip(self))]
    async fn position_update(&self, position: Position) -> Result<(), PortfolioError> {
        self.reconcilliate_position(position);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn fill_update(&self, fill: Fill) -> Result<(), PortfolioError> {
        self.update_position(fill);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn balance_update(&self, holding: Holding) -> Result<(), PortfolioError> {
        self.update_balance(holding);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn balances(&self) -> Vec<Holding> {
        self.balances.iter().map(|v| v.clone()).collect()
    }

    #[instrument(skip(self))]
    async fn positions(&self) -> Vec<Position> {
        self.positions.iter().map(|v| v.clone()).collect()
    }

    #[instrument(skip(self))]
    async fn capital(&self) -> Notional {
        self.capital + self.total_pnl().await - self.total_commission().await
    }

    #[instrument(skip(self))]
    async fn buying_power(&self) -> Notional {
        self.capital * self.leverage - self.total_exposure().await
    }

    #[instrument(skip(self))]
    async fn total_exposure(&self) -> Notional {
        self.positions.iter().fold(Notional::default(), |acc, e| {
            acc + e.avg_open_price * e.quantity + e.realized_pnl
        })
    }

    #[instrument(skip(self))]
    async fn total_pnl(&self) -> Notional {
        Notional::ZERO
    }

    #[instrument(skip(self))]
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
