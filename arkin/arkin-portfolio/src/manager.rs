use std::collections::HashMap;

use arkin_common::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::debug;

use crate::config::PortfolioManagerConfig;

pub struct PortfolioManager {
    capital: Notional,
    leverage: Decimal,
    positions: PositionState,
}

#[derive(Default, Clone)]
pub struct PositionState {
    positions: HashMap<(StrategyId, Instrument), Position>,
}

impl PositionState {
    pub fn position(&self, strategy_id: &StrategyId, instrument: &Instrument) -> Option<&Position> {
        self.positions.get(&(strategy_id.clone(), instrument.clone()))
    }

    pub fn update(&mut self, position: Position) {
        self.positions
            .insert((position.strategy_id.clone(), position.instrument.clone()), position);
    }
}

impl PortfolioManager {
    pub fn from_config(config: &PortfolioManagerConfig) -> Self {
        Self {
            capital: config.initial_capital.into(),
            leverage: config.leverage,
            positions: PositionState::default(),
        }
    }

    pub fn snapshot(&self, timestamp: &OffsetDateTime) -> PortfolioSnapshot {
        let positions = self.positions.positions.values().cloned().collect::<Vec<_>>();
        PortfolioSnapshot::new(timestamp.to_owned(), positions)
    }

    pub fn update_position(
        &mut self,
        timestamp: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) {
        if let Some(position) = self.positions.position(&strategy_id, &instrument) {
            let mut updating_position = position.clone();
            // We create a new updated value for point T so we can do historical look ups
            let remaining_quantity = updating_position.update(timestamp, side, price, quantity, commission);
            debug!("Updated position: {}", position);

            if let Some(remaining) = remaining_quantity {
                updating_position = Position::new(
                    timestamp,
                    strategy_id,
                    instrument,
                    side,
                    price,
                    remaining,
                    commission * (remaining / quantity),
                );
            }

            self.positions.update(updating_position);
        } else {
            debug!("No position found, inserting new position");
            let new_position = Position::new(timestamp, strategy_id, instrument, side, price, quantity, commission);
            self.positions.update(new_position);
        }
    }

    pub fn position(&self, strategy_id: &StrategyId, instrument: &Instrument) -> Option<&Position> {
        self.positions.position(strategy_id, instrument)
    }

    pub fn initial_capital(&self) -> Notional {
        self.capital
    }

    pub fn leverage(&self) -> Decimal {
        self.leverage
    }
}
