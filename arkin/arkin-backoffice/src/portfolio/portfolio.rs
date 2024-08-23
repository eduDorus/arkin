use dashmap::DashMap;
use rust_decimal::Decimal;
use std::{collections::BTreeMap, sync::Arc};
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::{
    config::PortfolioManagerConfig,
    models::{
        ExecutionOrder, ExecutionStatus, Instrument, Notional, Position, PositionSnapshot, PositionStatus, StrategyId,
    },
    state::StateManager,
    utils::CompositeIndex,
};

pub struct PortfolioManager {
    state: Arc<StateManager>,
    positions: DashMap<(StrategyId, Instrument), BTreeMap<CompositeIndex, Position>>,
    initial_capital: Notional,
    leverage: Decimal,
    initial_margin: Decimal,
    maintenance_margin: Decimal,
}

impl PortfolioManager {
    pub fn from_config(state: Arc<StateManager>, config: &PortfolioManagerConfig) -> Self {
        Self {
            state,
            positions: DashMap::new(),
            initial_capital: config.initial_capital.into(),
            leverage: config.leverage,
            initial_margin: config.initial_margin,
            maintenance_margin: config.maintenance_margin,
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
    pub fn position_snapshot(&self, timestamp: &OffsetDateTime) -> PositionSnapshot {
        let positions = self
            .positions
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
            .collect();
        PositionSnapshot::new(timestamp.clone(), positions)
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         config::StateManagerConfig,
//         logging,
//         models::{ExecutionType, Fill, OrderSide, TimeInForce, Venue},
//     };
//     use rust_decimal_macros::dec;
//     use std::time::Duration;
//     use tracing::info;

//     #[test]
//     fn test_portfolio_manager() {
//         logging::init_test_tracing();

//         let state = Arc::new(StateManager::from_config(&StateManagerConfig::default()));
//         let config = PortfolioManagerConfig {
//             initial_capital: dec!(100000),
//             leverage: dec!(1),
//             initial_margin: dec!(0.15),
//             maintenance_margin: dec!(0.1),
//         };
//         let portfolio_manager = PortfolioManager::from_config(state, &config);

//         let instrument = Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into());
//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             1,
//             ExecutionType::Market,
//             OrderSide::Buy,
//             dec!(100).into(),
//             TimeInForce::Gtc,
//         );
//         portfolio_manager.update_position(order.clone());

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             1,
//             1234324232,
//             dec!(10).into(),
//             dec!(40).into(),
//             dec!(0.3).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(10),
//             instrument.clone(),
//             1,
//             1234324233,
//             dec!(20).into(),
//             dec!(40).into(),
//             dec!(0.5).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(5),
//             instrument.clone(),
//             1,
//             1234324233,
//             dec!(18).into(),
//             dec!(20).into(),
//             dec!(1.0).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             2,
//             ExecutionType::Market,
//             OrderSide::Sell,
//             dec!(100).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             2,
//             1234324235,
//             dec!(30).into(),
//             dec!(100).into(),
//             dec!(2.3).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             3,
//             ExecutionType::Market,
//             OrderSide::Sell,
//             dec!(50).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             3,
//             1234324235,
//             dec!(30).into(),
//             dec!(50).into(),
//             dec!(1.3).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             4,
//             ExecutionType::Market,
//             OrderSide::Buy,
//             dec!(50).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             4,
//             1234324235,
//             dec!(20).into(),
//             dec!(50).into(),
//             dec!(1.0).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             5,
//             ExecutionType::Market,
//             OrderSide::Buy,
//             dec!(50).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             5,
//             1234324235,
//             dec!(20).into(),
//             dec!(50).into(),
//             dec!(1.0).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             6,
//             ExecutionType::Market,
//             OrderSide::Sell,
//             dec!(100).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             6,
//             1234324235,
//             dec!(15).into(),
//             dec!(100).into(),
//             dec!(1.0).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         let mut order = ExecutionOrder::new(
//             OffsetDateTime::now_utc(),
//             "Strategy1".into(),
//             instrument.clone(),
//             6,
//             ExecutionType::Market,
//             OrderSide::Buy,
//             dec!(50).into(),
//             TimeInForce::Gtc,
//         );

//         let fill = Fill::new(
//             OffsetDateTime::now_utc() - Duration::from_secs(15),
//             instrument.clone(),
//             6,
//             1234324235,
//             dec!(10).into(),
//             dec!(50).into(),
//             dec!(1.0).into(),
//         );
//         debug!("Adding fill: {}", fill);
//         order.update(&fill);
//         debug!("Order: {}", order);
//         portfolio_manager.update_position(order.clone());

//         // Print all positions
//         portfolio_manager
//             .positions
//             .iter()
//             .map(|v| v.value().clone())
//             .flatten()
//             .for_each(|v| info!("Position: {}", v.1))
//     }
// }
