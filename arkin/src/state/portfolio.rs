use std::{collections::BTreeMap, ops::Not, time::Duration};

use dashmap::DashMap;
use time::OffsetDateTime;

use crate::{
    config::PortfolioStateConfig,
    models::{ExecutionOrder, ExecutionStatus, Instrument, Notional, Position, PositionStatus},
    strategies::StrategyId,
    utils::CompositeIndex,
};

// The hirarchy for positions is as followed:

pub struct PortfolioState {
    initial_capital: Notional,
    execution_orders: DashMap<(StrategyId, Instrument), BTreeMap<CompositeIndex, ExecutionOrder>>,
    internal_trades: DashMap<(StrategyId, Instrument), BTreeMap<CompositeIndex, Position>>,
}

impl PortfolioState {
    pub fn from_config(config: &PortfolioStateConfig) -> Self {
        Self {
            initial_capital: config.initial_capital.into(),
            execution_orders: DashMap::new(),
            internal_trades: DashMap::new(),
        }
    }

    pub fn add_execution_order(&self, order: &ExecutionOrder) {
        // Creste the key to access the DashMap
        let key = (order.strategy_id.clone(), order.instrument.clone());
        let mut composit_key = CompositeIndex::new(&order.last_updated_at);

        let mut entry = self.execution_orders.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, order.clone());

        if order.status == ExecutionStatus::PartiallyFilled || order.status == ExecutionStatus::Filled {
            self.update_internal_trades(order);
        }
    }

    fn update_internal_trades(&self, order: &ExecutionOrder) {
        // Creste the key to access the DashMap
        let key = (order.strategy_id.clone(), order.instrument.clone());

        // Get or create a BTreeMap for the strategy and instrument
        let mut trades_entry = self.internal_trades.entry(key.clone()).or_default();

        // Obtain mutable access to the BTreeMap within DashMap
        let trades_entry = trades_entry.value_mut();

        // Get the last trade if it exists and is open, otherwise start a new trade
        let trade = if let Some((_, last_trade)) = trades_entry.iter_mut().last() {
            if last_trade.status == PositionStatus::Open {
                // Update the existing trade
                last_trade.avg_open_price = (last_trade.avg_open_price * last_trade.quantity
                    + order.avg_price * order.quantity)
                    / (last_trade.quantity + order.quantity);
                last_trade.quantity += order.quantity;
                last_trade.commission = order.commission;
                last_trade.clone() // Clone it for further use
            } else {
                // If the last trade isn't open, start a new one
                Position::from(order.to_owned())
            }
        } else {
            // If there are no trades, start a new one
            Position::from(order.to_owned())
        };

        // Generate a new composite key for the trade
        let mut composite_key = CompositeIndex::new(&trade.created_at);

        // Ensure the composite key is unique within the BTreeMap
        while trades_entry.contains_key(&composite_key) {
            composite_key.increment();
        }

        // Insert the new or updated trade back into the BTreeMap
        trades_entry.insert(composite_key, trade);
    }

    fn open_execution_orders(&self, timestamp: &OffsetDateTime) -> impl Iterator<Item = ExecutionOrder> + '_ {
        let index = CompositeIndex::new_max(timestamp);
        self.execution_orders.iter().flat_map(move |entry| {
            entry
                .value()
                .range(..=index)
                .filter_map(|(_, order)| {
                    if order.is_active() {
                        Some(order.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
    }

    fn closed_execution_orders(&self, timestamp: &OffsetDateTime) -> impl Iterator<Item = ExecutionOrder> + '_ {
        let index = CompositeIndex::new_max(timestamp);
        self.execution_orders.iter().flat_map(move |entry| {
            entry
                .value()
                .range(..=index)
                .filter_map(|(_, order)| {
                    if order.is_active().not() {
                        Some(order.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
    }

    // Since we know that the only option for a open position can be the last position in the BTreeMap we will just check that and clone if true
    fn find_open_positions(&self, timestamp: &OffsetDateTime) -> impl Iterator<Item = Position> + '_ {
        let index = CompositeIndex::new_max(timestamp);
        self.internal_trades
            .iter()
            .flat_map(move |entry| {
                entry.value().range(..=index).next_back().map(|(_, p)| {
                    if p.status == PositionStatus::Open {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
            })
            .flatten()
    }

    fn closed_positions(&self, timestamp: &OffsetDateTime) -> impl Iterator<Item = Position> + '_ {
        let index = CompositeIndex::new_max(timestamp);
        self.internal_trades.iter().flat_map(move |entry| {
            entry
                .value()
                .range(..=index)
                .filter_map(|(_, trade)| {
                    if trade.status == PositionStatus::Closed {
                        Some(trade.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
    }

    pub fn capital(&self) -> Notional {
        self.initial_capital
    }

    // pub fn total_value(&self, market_data: &MarketData, timestamp: &OffsetDateTime) -> Notional {
    //     self.capital + self.total_unrealized_pnl(market_data, timestamp)
    // }

    // pub fn total_exposure(&self, as_of: OffsetDateTime) -> Decimal {
    //     self.calculate_positions(None, None, as_of).values().map(|p| p.exposure()).sum()
    // }

    pub fn total_realized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
        self.closed_positions(timestamp).map(|t| t.realized_pnl).sum()
    }

    // pub fn totoal_unrealized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
    //     self.find_open_positions(timestamp).map(|p| p.unrealized_pnl()).sum()
    // }

    pub fn open_orders_count(&self, timestamp: &OffsetDateTime) -> usize {
        self.open_execution_orders(timestamp).count()
    }

    pub fn average_fill_time(&self, timestamp: &OffsetDateTime) -> Option<Duration> {
        let fill_times: Vec<Duration> = self.closed_execution_orders(timestamp).filter_map(|o| o.fill_time()).collect();

        if fill_times.is_empty() {
            None
        } else {
            Some(fill_times.iter().sum::<Duration>() / fill_times.len() as u32)
        }
    }

    pub fn positions(&self, timestamp: &OffsetDateTime) -> Vec<Position> {
        self.find_open_positions(timestamp).collect()
    }

    // pub fn strategy_performance(&self, strategy_id: &StrategyId, as_of: OffsetDateTime) -> StrategyPerformance {
    //     let positions = self.calculate_positions(Some(strategy_id), None, as_of);
    //     let realized_pnl = self
    //         .internal_trades
    //         .iter()
    //         .filter(|t| t.strategy_id == *strategy_id && t.open_time <= as_of)
    //         .map(|t| t.realized_pnl)
    //         .sum();

    //     StrategyPerformance {
    //         positions,
    //         realized_pnl,
    //     }
    // }

    // pub fn rejection_rate(&self, start: OffsetDateTime, end: OffsetDateTime) -> f64 {
    //     let orders: Vec<_> = self.execution_orders.range(start..=end).map(|(_, order)| order).collect();
    //     let total_orders = orders.len();
    //     let rejected_orders = orders.iter().filter(|order| order.status == ExecutionStatus::Rejected).count();

    //     if total_orders > 0 {
    //         rejected_orders as f64 / total_orders as f64
    //     } else {
    //         0.0
    //     }
    // }
}

// pub fn capital(&self) -> &Notional {
//     &self.capital
// }

// pub fn buying_power(&self, event_time: &OffsetDateTime) -> Notional {
//     self.capital - self.total_exposure(event_time)
// }

// pub fn total_exposure(&self, event_time: &OffsetDateTime) -> Notional {
//     let positions = self.positions(event_time);
//     positions
//         .values()
//         .map(|p| p.quantity * p.avg_price)
//         .fold(Notional::from(0.), |acc, x| acc + x)
// }

// pub fn absolut_exposure(&self, event_time: &OffsetDateTime) -> Notional {
//     let positions = self.positions(event_time);
//     positions
//         .values()
//         .map(|p| p.quantity.abs() * p.avg_price)
//         .fold(Notional::from(0.), |acc, x| acc + x)
// }

// pub fn positions(&self, timestamp: &OffsetDateTime) -> HashMap<(StrategyId, Instrument), Position> {
//     let fills = self.state.events::<Fill>(timestamp);
//     fills.iter().map(|(_, f)| f).flatten().for_each(|f| debug!("Fill: {}", f));

//     let strategies_instruments = fills
//         .values()
//         .flatten()
//         .map(|f| (f.strategy_id.clone(), f.instrument.clone()))
//         .collect::<HashSet<(StrategyId, Instrument)>>();
//     for (s, i) in &strategies_instruments {
//         debug!("Strategy: {}, Instrument: {}", s, i);
//     }

//     let positions = strategies_instruments.into_iter().fold(HashMap::new(), |mut acc, (s, i)| {
//         let fills = fills.get(&i).unwrap().iter().filter(|f| f.strategy_id == s).collect::<Vec<_>>();
//         if let Some(position) = self.calculate_trades_from_fills(fills).last() {
//             acc.insert((s, i), position.to_owned());
//         }
//         acc
//     });

//     positions.iter().for_each(|(k, v)| {
//         debug!("Strategy: {}, Instrument: {}, Position: {}", k.0, k.1, v);
//     });
//     positions
// }

// pub fn all_positions(&self, timestamp: &OffsetDateTime) -> HashMap<(StrategyId, Instrument), Vec<Position>> {
//     let fills = self.state.events::<Fill>(timestamp);
//     fills.iter().map(|(_, f)| f).flatten().for_each(|f| debug!("Fill: {}", f));

//     let strategies_instruments = fills
//         .values()
//         .flatten()
//         .map(|f| (f.strategy_id.clone(), f.instrument.clone()))
//         .collect::<HashSet<(StrategyId, Instrument)>>();

//     for (s, i) in &strategies_instruments {
//         debug!("Strategy: {}, Instrument: {}", s, i);
//     }

//     let positions = strategies_instruments.into_iter().fold(HashMap::new(), |mut acc, (s, i)| {
//         let fills = fills.get(&i).unwrap().iter().filter(|f| f.strategy_id == s).collect::<Vec<_>>();
//         let trades = self.calculate_trades_from_fills(fills);
//         if let Some(position) = trades.last() {
//             acc.insert((s, i), trades);
//         }
//         acc.insert((s, i), trades);
//         acc
//     });

//     positions.iter().for_each(|(k, v)| {
//         debug!("Strategy: {}, Instrument: {}", k.0, k.1);
//         v.iter().for_each(|p| debug!("{}", p));
//     });
//     positions
// }

// fn calculate_trades_from_fills(&self, fills: Vec<&Fill>) -> Vec<Position> {
//     let mut trades = Vec::new();
//     let mut current_trade = Option::<Position>::None;
//     for fill in fills {
//         // Fill the position
//         let (excess, position) = match current_trade {
//             None => (None, Position::from_fill(fill)),
//             Some(mut p) => {
//                 let excess = p.update(fill);
//                 (excess, p)
//             }
//         };
//         if let Some(e) = excess {
//             trades.push(position);
//             current_trade = Some(Position::from_fill(&e));
//         } else {
//             current_trade = Some(position);
//         }
//     }
//     if let Some(p) = current_trade {
//         trades.push(p);
//     }
//     trades
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{logging, test_utils};
//     use time::macros::datetime;
//     use tracing::info;

//     #[test]
//     fn test_portfolio() {
//         logging::init_test_tracing();

//         let instrument = test_utils::test_multi_perp_instrument();
//         let state = test_utils::TestStateBuilder::default()
//             .add_fills(&instrument[0])
//             // .add_fills(&instrument[1])
//             .build();

//         let portfolio = PortfolioState::new(state, Notional::from(2000.));

//         let mut event_time = datetime!(2024-01-01 00:00:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(0.));
//         // assert_eq!(position.quantity, Quantity::from(0.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(2000.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(0.));

//         event_time = datetime!(2024-01-01 00:01:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(80.));
//         // assert_eq!(position.quantity, Quantity::from(10.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(1200.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(800.));

//         event_time = datetime!(2024-01-01 00:02:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(100.));
//         // assert_eq!(position.quantity, Quantity::from(20.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(0.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(2000.));

//         event_time = datetime!(2024-01-01 00:03:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(100.));
//         // assert_eq!(position.quantity, Quantity::from(10.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(1000.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(1000.));

//         event_time = datetime!(2024-01-01 00:04:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(100.));
//         // assert_eq!(position.quantity, Quantity::from(-10.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(3000.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(-1000.));

//         event_time = datetime!(2024-01-01 00:05:00).assume_utc();
//         for ((s, i), v) in portfolio.positions(&event_time).iter() {
//             info!("{}: {}: {}", s, i, v);
//         }
//         // assert_eq!(position.avg_price, Price::from(0.));
//         // assert_eq!(position.quantity, Quantity::from(0.));
//         assert_eq!(portfolio.buying_power(&event_time), Notional::from(2750.));
//         assert_eq!(portfolio.total_exposure(&event_time), Notional::from(-750.));
//     }
// }
