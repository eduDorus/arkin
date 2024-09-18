use arkin_core::prelude::*;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::info;

use crate::config::PortfolioManagerConfig;

pub struct PortfolioManager {
    capital: Notional,
    leverage: Decimal,
    positions: DashMap<(StrategyId, Instrument), Position>,
    position_history: DashMap<(StrategyId, Instrument), Vec<Position>>,
}

impl PortfolioManager {
    pub fn new(initial_capital: Notional, leverage: Decimal) -> Self {
        Self {
            capital: initial_capital,
            leverage,
            positions: DashMap::new(),
            position_history: DashMap::new(),
        }
    }

    pub fn from_config(config: &PortfolioManagerConfig) -> Self {
        Self::new(config.initial_capital, config.leverage)
    }

    pub fn update_position_from_fill(&self, fill: Fill) {
        self.update_position(
            fill.created_at,
            fill.strategy_id,
            fill.instrument,
            fill.side,
            fill.price,
            fill.quantity,
            fill.commission,
        );
    }

    pub fn update_position(
        &self,
        event_time: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) {
        let key = (strategy_id.clone(), instrument.clone());

        if let Some(mut position) = self.positions.get_mut(&key) {
            let (decreasing, increasing) = match (position.side, side) {
                (PositionSide::Long, Side::Sell) | (PositionSide::Short, Side::Buy) => (true, false),
                _ => (false, true),
            };

            if increasing {
                self.increase_position(&mut position, price, quantity, commission, event_time);
            } else if decreasing {
                let remaining = self.decrease_position(&mut position, price, quantity, commission, event_time);
                if position.quantity.is_zero() {
                    // Append the position to the history
                    self.position_history
                        .entry(key.clone())
                        .or_insert(Vec::new())
                        .push(position.clone());

                    // Release the mutable reference
                    drop(position);
                    self.positions.remove(&key);
                }
                if let Some(remaining_quantity) = remaining {
                    self.create_new_position(
                        event_time,
                        strategy_id,
                        instrument,
                        side,
                        price,
                        remaining_quantity,
                        Notional::default(),
                    );
                }
            }
        } else {
            self.create_new_position(event_time, strategy_id, instrument, side, price, quantity, commission);
        }
    }

    fn increase_position(
        &self,
        position: &mut Position,
        price: Price,
        quantity: Quantity,
        commission: Notional,
        event_time: OffsetDateTime,
    ) {
        let new_total_quantity = position.quantity + quantity;
        position.avg_open_price = (position.avg_open_price * position.quantity + price * quantity) / new_total_quantity;
        position.quantity = new_total_quantity;
        position.trade_volume += price * quantity;
        position.commission += commission;
        position.updated_at = event_time;

        info!("Increased position: {}", position);
    }

    fn decrease_position(
        &self,
        position: &mut Position,
        price: Price,
        quantity: Quantity,
        commission: Notional,
        event_time: OffsetDateTime,
    ) -> Option<Quantity> {
        let close_quantity = position.quantity.min(quantity);
        let pnl = self.calculate_pnl(position, price, close_quantity);

        position.realized_pnl += pnl;
        position.trade_volume += price * close_quantity;
        position.quantity -= close_quantity;
        position.avg_close_price = price;
        position.commission += commission;
        position.updated_at = event_time;

        if position.quantity.is_zero() {
            position.status = PositionStatus::Closed;
        }
        info!("Reduced position: {}", position);

        if close_quantity < quantity {
            Some(quantity - close_quantity)
        } else {
            None
        }
    }

    fn create_new_position(
        &self,
        event_time: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) {
        let new_position = Position {
            strategy_id: strategy_id.clone(),
            instrument: instrument.clone(),
            side: match side {
                Side::Buy => PositionSide::Long,
                Side::Sell => PositionSide::Short,
            },
            avg_open_price: price,
            avg_close_price: Price::default(),
            quantity,
            trade_volume: price * quantity,
            realized_pnl: Notional::default(),
            commission,
            status: PositionStatus::Open,
            created_at: event_time,
            updated_at: event_time,
        };

        info!("Created new position: {}", new_position);
        self.positions.insert((strategy_id, instrument), new_position);
    }

    fn calculate_pnl(&self, position: &Position, close_price: Price, close_quantity: Quantity) -> Notional {
        match position.side {
            PositionSide::Long => Price::from(close_price - position.avg_open_price) * close_quantity,
            PositionSide::Short => Price::from(position.avg_open_price - close_price) * close_quantity,
        }
    }

    pub fn open_position(&self, strategy_id: &StrategyId, instrument: &Instrument) -> Option<Position> {
        self.positions.get(&(strategy_id.clone(), instrument.clone())).and_then(|v| {
            if v.value().status == PositionStatus::Open {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    pub fn snapshot(&self, timestamp: &OffsetDateTime) -> PortfolioSnapshot {
        let positions = self.positions.iter().map(|v| v.clone()).collect();
        PortfolioSnapshot::new(timestamp.to_owned(), self.capital, positions)
    }

    pub fn available_capital(&self) -> Notional {
        self.capital + self.total_pnl() - self.total_commission()
    }

    pub fn leverage(&self) -> Decimal {
        self.leverage
    }

    pub fn print_stats(&self) {
        info!("");
        info!("------------------ Portfolio Stats --------------------");
        info!("Capital: {}", self.available_capital().round_dp(2));
        info!("Return %: {}", self.return_pct().round_dp(5));
        info!("Profit: {}", self.profit().round_dp(2));
        info!("Loss: {}", self.loss().round_dp(2));
        info!("PnL: {}", self.total_pnl().round_dp(2));
        info!("Commission: {}", self.total_commission().round_dp(2));

        info!("");
        info!("Overall Stats:");
        info!("Trade Volume: {}", self.total_trade_volume().round_dp(2));
        info!("Trade Count: {}", self.total_trades());
        info!("Avg Trade Return: {}", self.avg_return_per_trade().round_dp(2));
        info!("Avg Trade Return Pct: {}", self.avg_return_per_trade_pct().round_dp(5));
        info!("");
        info!("Win Trades:");
        info!("Count: {}", self.total_win_trades());
        info!("Avg Return: {}", self.avg_return_per_win_trade().round_dp(2));
        info!("Max Return: {}", self.max_profit_trade().round_dp(2));
        info!("Avg Return Pct: {}", self.avg_return_per_win_trade_pct().round_dp(5));
        info!("Max Return Pct: {}", self.max_profit_trade_pct().round_dp(5));
        info!("");
        info!("Loss Trades:");
        info!("Count: {}", self.total_loss_trades());
        info!("Avg Return: {}", self.avg_loss_per_trade().round_dp(2));
        info!("Max Return: {}", self.max_loss_trade().round_dp(2));
        info!("Avg Return Pct: {}", self.avg_loss_per_trade_pct().round_dp(5));
        info!("Max Return Pct: {}", self.max_loss_trade_pct().round_dp(5));
        info!("");
        info!("Advanced Stats:");
        info!("Win Rate: {}", self.win_rate().round_dp(2));
        info!("Profit Factor: {}", self.profit_factor().round_dp(2));
    }

    pub fn print_trades(&self) {
        info!("------------------ Portfolio Trades --------------------");
        for entry in self.position_history.iter() {
            for position in entry.value() {
                info!("- {}", position);
            }
        }
    }

    pub fn return_pct(&self) -> Decimal {
        let total_pnl = self.total_pnl();
        let total_commission = self.total_commission();

        (self.capital + total_pnl - total_commission) / self.capital
    }

    pub fn profit(&self) -> Notional {
        self.position_history.iter().fold(Notional::default(), |acc, e| {
            acc + e.value().iter().fold(Notional::default(), |acc, p| {
                if p.is_profitable() {
                    acc + p.realized_pnl
                } else {
                    acc
                }
            })
        })
    }

    pub fn loss(&self) -> Notional {
        self.position_history.iter().fold(Notional::default(), |acc, e| {
            acc + e.value().iter().fold(Notional::default(), |acc, p| {
                if !p.is_profitable() {
                    acc + p.realized_pnl
                } else {
                    acc
                }
            })
        })
    }

    pub fn total_pnl(&self) -> Notional {
        self.position_history.iter().fold(Notional::default(), |acc, e| {
            acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.realized_pnl)
        })
    }

    pub fn total_commission(&self) -> Notional {
        self.position_history.iter().fold(Notional::default(), |acc, e| {
            acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.commission)
        })
    }

    pub fn total_trade_volume(&self) -> Notional {
        self.position_history.iter().fold(Notional::default(), |acc, e| {
            acc + e.value().iter().fold(Notional::default(), |acc, p| acc + p.trade_volume)
        })
    }

    pub fn total_trades(&self) -> usize {
        self.position_history.iter().fold(0, |acc, e| acc + e.value().len())
    }

    pub fn total_win_trades(&self) -> usize {
        self.position_history.iter().fold(0, |acc, e| {
            acc + e.value().iter().fold(0, |acc, p| if p.is_profitable() { acc + 1 } else { acc })
        })
    }

    pub fn total_loss_trades(&self) -> usize {
        self.position_history.iter().fold(0, |acc, e| {
            acc + e
                .value()
                .iter()
                .fold(0, |acc, p| if !p.is_profitable() { acc + 1 } else { acc })
        })
    }

    pub fn win_rate(&self) -> Decimal {
        let win_trades = self.total_win_trades();
        let trade_count = self.total_trades();
        if trade_count == 0 {
            Decimal::ZERO
        } else {
            Decimal::from(win_trades) / Decimal::from(trade_count)
        }
    }

    pub fn avg_return_per_trade(&self) -> Notional {
        let total_pnl = self.total_pnl();
        let total_trades = self.total_trades();
        if total_trades == 0 {
            Notional::default()
        } else {
            total_pnl / Decimal::from(total_trades)
        }
    }

    pub fn avg_return_per_trade_pct(&self) -> Decimal {
        self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
            acc + e.value().iter().fold(Decimal::ZERO, |acc, p| acc + p.return_pct())
        }) / Decimal::from(self.total_trades())
    }

    pub fn avg_return_per_win_trade(&self) -> Decimal {
        let profit = self.profit();
        let total_win_trades = self.total_win_trades();
        if total_win_trades == 0 {
            Decimal::ZERO
        } else {
            profit / Decimal::from(total_win_trades)
        }
    }

    pub fn avg_return_per_win_trade_pct(&self) -> Decimal {
        let total_win_trades = self.total_win_trades();
        if total_win_trades == 0 {
            Decimal::ZERO
        } else {
            self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
                acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
                    if p.is_profitable() {
                        acc + p.return_pct()
                    } else {
                        acc
                    }
                })
            }) / Decimal::from(total_win_trades)
        }
    }

    pub fn avg_return_per_loss_trade(&self) -> Decimal {
        let loss = self.loss();
        let total_loss_trades = self.total_loss_trades();
        if total_loss_trades == 0 {
            Decimal::ZERO
        } else {
            loss / Decimal::from(total_loss_trades)
        }
    }

    pub fn avg_return_per_loss_trade_pct(&self) -> Decimal {
        let total_loss_trades = self.total_loss_trades();
        if total_loss_trades == 0 {
            Decimal::ZERO
        } else {
            self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
                acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
                    if !p.is_profitable() {
                        acc + p.return_pct()
                    } else {
                        acc
                    }
                })
            }) / Decimal::from(total_loss_trades)
        }
    }

    pub fn max_profit_trade(&self) -> Notional {
        let mut max_profit = Decimal::ZERO;
        self.position_history.iter().for_each(|e| {
            e.value().iter().for_each(|p| {
                if p.is_profitable() && p.realized_pnl > max_profit {
                    max_profit = p.realized_pnl;
                }
            });
        });
        max_profit
    }

    pub fn max_profit_trade_pct(&self) -> Decimal {
        let mut max_profit = Decimal::ZERO;
        self.position_history.iter().for_each(|e| {
            e.value().iter().for_each(|p| {
                if p.is_profitable() && p.return_pct() > max_profit {
                    max_profit = p.return_pct();
                }
            });
        });
        max_profit
    }

    pub fn avg_loss_per_trade(&self) -> Decimal {
        let loss = self.loss();
        let total_loss_trades = self.total_loss_trades();
        if total_loss_trades == 0 {
            Decimal::ZERO
        } else {
            loss / Decimal::from(total_loss_trades)
        }
    }

    pub fn avg_loss_per_trade_pct(&self) -> Decimal {
        let total_loss_trades = self.total_loss_trades();
        if total_loss_trades == 0 {
            Decimal::ZERO
        } else {
            self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
                acc + e.value().iter().fold(Decimal::ZERO, |acc, p| {
                    if !p.is_profitable() {
                        acc + p.return_pct()
                    } else {
                        acc
                    }
                })
            }) / Decimal::from(total_loss_trades)
        }
    }

    pub fn max_loss_trade(&self) -> Notional {
        let mut max_loss = Decimal::ZERO;
        self.position_history.iter().for_each(|e| {
            e.value().iter().for_each(|p| {
                if !p.is_profitable() && p.realized_pnl.abs() > max_loss {
                    max_loss = p.realized_pnl.abs();
                }
            });
        });
        -max_loss
    }

    pub fn max_loss_trade_pct(&self) -> Decimal {
        let mut max_loss = Decimal::ZERO;
        self.position_history.iter().for_each(|e| {
            e.value().iter().for_each(|p| {
                if !p.is_profitable() && p.return_pct().abs() > max_loss {
                    max_loss = p.return_pct().abs();
                }
            });
        });
        -max_loss
    }

    pub fn profit_factor(&self) -> Decimal {
        let profit = self.profit();
        let loss = self.loss().abs();
        if loss == Decimal::ZERO {
            Decimal::MAX
        } else {
            profit / loss
        }
    }
}
