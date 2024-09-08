use arkin_common::prelude::*;
use dashmap::DashMap;
use rust_decimal::Decimal;
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

    pub fn stats(&self) {
        let trade_count = self.position_history.iter().fold(0, |acc, e| acc + e.value().len());
        let pnl = self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
            acc + e.value().iter().fold(Decimal::ZERO, |acc, p| acc + p.realized_pnl)
        });
        let commission = self.position_history.iter().fold(Decimal::ZERO, |acc, e| {
            acc + e.value().iter().fold(Decimal::ZERO, |acc, p| acc + p.commission)
        });
        info!("Traded {trade_count} times with PnL {pnl} and commission {commission}");
        for entry in self.position_history.iter() {
            let strategy = entry.key().0.clone();
            let instrument = entry.key().1.clone();
            let positions = entry.value();
            info!("Strategy {strategy} with instrument {instrument}");
            for position in positions {
                info!("- {}", position);
            }
        }
    }

    pub fn from_config(config: &PortfolioManagerConfig) -> Self {
        Self::new(config.initial_capital, config.leverage)
    }

    pub fn update_position_from_fill(&self, fill: Fill) {
        self.update_position(
            fill.event_time,
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
        position.total_quantity += quantity;
        position.commission += commission;
        position.last_updated_at = event_time;

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
        position.quantity -= close_quantity;
        position.avg_close_price = price;
        position.commission += commission;
        position.last_updated_at = event_time;

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
            total_quantity: quantity,
            realized_pnl: Notional::default(),
            commission,
            status: PositionStatus::Open,
            created_at: event_time,
            last_updated_at: event_time,
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
        self.capital
    }

    pub fn leverage(&self) -> Decimal {
        self.leverage
    }
}
