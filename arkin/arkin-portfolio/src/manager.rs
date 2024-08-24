use std::collections::BTreeMap;

use arkin_common::prelude::*;
use dashmap::DashMap;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::config::PortfolioManagerConfig;

pub struct PortfolioManager {
    positions: DashMap<(StrategyId, Instrument), BTreeMap<CompositeIndex, Position>>,
    initial_capital: Notional,
    leverage: Decimal,
    _initial_margin: Decimal,
    _maintenance_margin: Decimal,
}

impl PortfolioManager {
    pub fn from_config(config: &PortfolioManagerConfig) -> Self {
        Self {
            positions: DashMap::new(),
            initial_capital: config.initial_capital.into(),
            leverage: config.leverage,
            _initial_margin: config.initial_margin,
            _maintenance_margin: config.maintenance_margin,
        }
    }

    pub fn update_position(&self, order: ExecutionOrder) {
        if !self.is_valid_position_update(&order) {
            warn!("Invalid position update: {}", order);
            return;
        }

        let key = (order.strategy_id.clone(), order.instrument.clone());
        self.positions
            .entry(key)
            .and_modify(|position_tree| {
                if let Some(entry) = position_tree.last_entry() {
                    if entry.get().status == PositionStatus::Open {
                        debug!("Updating position: {}", entry.get());

                        // We create a new updated value for point T so we can do historical look ups
                        let mut position = entry.get().clone();
                        let remaining_quantity = position.update_with_order(&order);
                        debug!("Updated position: {}", position);
                        let new_index = CompositeIndex::new(&position.last_updated_at);
                        position_tree.insert(new_index, position);

                        if let Some(quantity) = remaining_quantity {
                            let mut remaining_order = order.to_owned();
                            remaining_order.last_fill_quantity = quantity;
                            self.insert_new_position(position_tree, &remaining_order);
                        }
                    } else {
                        debug!("No open position, inserting new position: {}", order);
                        self.insert_new_position(position_tree, &order);
                    }
                } else {
                    debug!("No last entry, inserting new position: {}", order);
                    self.insert_new_position(position_tree, &order);
                }
            })
            .or_insert_with(|| {
                debug!("No position found, inserting new position: {}", order);
                let mut new_tree = BTreeMap::new();
                self.insert_new_position(&mut new_tree, &order);
                new_tree
            });
    }

    fn is_valid_position_update(&self, order: &ExecutionOrder) -> bool {
        matches!(order.status, ExecutionStatus::PartiallyFilled | ExecutionStatus::Filled)
    }

    fn insert_new_position(&self, tree: &mut BTreeMap<CompositeIndex, Position>, order: &ExecutionOrder) {
        let new_position = Position::from(order.clone());
        let new_index = CompositeIndex::new(&new_position.last_updated_at);
        tree.insert(new_index, new_position);
    }

    /// Get the latest position snapshot at a given timestamp (take the last position before the timestamp)
    pub fn snapshot(&self, timestamp: &OffsetDateTime) -> Vec<Position> {
        self.positions
            .iter()
            .map(|v| v.value().clone())
            .filter_map(|v| {
                let position = v.values().rev().find(|p| p.last_updated_at <= *timestamp);
                if let Some(p) = position {
                    // Check if position is open and return it
                    if p.status == PositionStatus::Open {
                        return Some(p.clone());
                    }
                }
                None
            })
            .collect()
    }

    pub fn total_capital(&self) -> Notional {
        self.initial_capital
    }

    pub fn leverage(&self) -> Decimal {
        self.leverage
    }

    pub fn buying_power(&self) -> Notional {
        self.total_capital() * self.leverage()
    }

    // pub fn total_value(&self, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_value_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_value_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_exposure(&self, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_exposure_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_exposure_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_realized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_realized_pnl_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_realized_pnl_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_unrealized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_unrealized_pnl_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }

    // pub fn total_unrealized_pnl_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
    //     todo!()
    // }
}
