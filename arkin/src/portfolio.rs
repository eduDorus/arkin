use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use time::OffsetDateTime;

use crate::{
    models::{Fill, Instrument, Notional, Position},
    state::StateManager,
    strategies::StrategyId,
};

// The hirarchy for positions is as followed:

pub struct Portfolio {
    state: Arc<StateManager>,
    capital: Notional,
}

impl Portfolio {
    pub fn new(state: Arc<StateManager>, capital: Notional) -> Self {
        Self { state, capital }
    }
}

impl Portfolio {
    pub fn capital(&self) -> &Notional {
        &self.capital
    }

    pub fn buying_power(&self, event_time: &OffsetDateTime) -> Notional {
        self.capital - self.total_exposure(event_time)
    }

    pub fn total_exposure(&self, event_time: &OffsetDateTime) -> Notional {
        let positions = self.positions(event_time);
        positions
            .values()
            .map(|p| p.quantity.abs() * p.avg_price)
            .fold(Notional::from(0.), |acc, x| acc + x)
    }

    pub fn positions(&self, timestamp: &OffsetDateTime) -> HashMap<(StrategyId, Instrument), Position> {
        let fills = self.state.events::<Fill>(timestamp);

        let strategies_instruments = fills
            .values()
            .flatten()
            .map(|f| (f.strategy_id.clone(), f.instrument.clone()))
            .collect::<HashSet<(StrategyId, Instrument)>>();

        strategies_instruments.into_iter().fold(HashMap::new(), |mut acc, (s, i)| {
            let fills = fills.get(&i).unwrap().iter().filter(|f| f.strategy_id == s).collect::<Vec<_>>();
            if let Some(position) = self.calculate_positions_from_fills(fills).last() {
                acc.insert((s, i), position.to_owned());
            }
            acc
        })
    }

    pub fn all_positions(&self, timestamp: &OffsetDateTime) -> HashMap<(StrategyId, Instrument), Vec<Position>> {
        let fills = self.state.events::<Fill>(timestamp);

        let strategies_instruments = fills
            .values()
            .flatten()
            .map(|f| (f.strategy_id.clone(), f.instrument.clone()))
            .collect::<HashSet<(StrategyId, Instrument)>>();

        strategies_instruments.into_iter().fold(HashMap::new(), |mut acc, (s, i)| {
            let fills = fills.get(&i).unwrap().iter().filter(|f| f.strategy_id == s).collect::<Vec<_>>();
            let positions = self.calculate_positions_from_fills(fills);
            acc.insert((s, i), positions);
            acc
        })
    }

    fn calculate_positions_from_fills(&self, fills: Vec<&Fill>) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut current_position = Option::<Position>::None;
        for fill in fills {
            // Fill the position
            let (excess, position) = match current_position {
                None => (None, Position::from_fill(fill)),
                Some(mut p) => {
                    let excess = p.update(fill);
                    (excess, p)
                }
            };
            if excess.is_some() {
                positions.push(position);
                current_position = None;
            } else {
                current_position = Some(position);
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{logging, test_utils};
    use time::macros::datetime;
    use tracing::info;

    #[test]
    fn test_portfolio() {
        logging::init_test_tracing();

        let instrument = test_utils::test_multi_perp_instrument();
        let state = test_utils::TestStateBuilder::default()
            .add_fills(&instrument[0])
            // .add_fills(&instrument[1])
            .build();

        let portfolio = Portfolio::new(state, Notional::from(2000.));

        let mut event_time = datetime!(2024-01-01 00:00:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(0.));
        // assert_eq!(position.quantity, Quantity::from(0.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(2000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(0.));

        event_time = datetime!(2024-01-01 00:01:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(80.));
        // assert_eq!(position.quantity, Quantity::from(10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(1200.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(800.));

        event_time = datetime!(2024-01-01 00:02:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(100.));
        // assert_eq!(position.quantity, Quantity::from(20.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(0.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(2000.));

        event_time = datetime!(2024-01-01 00:03:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(100.));
        // assert_eq!(position.quantity, Quantity::from(10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(1000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(1000.));

        event_time = datetime!(2024-01-01 00:04:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(100.));
        // assert_eq!(position.quantity, Quantity::from(-10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(3000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(-1000.));

        event_time = datetime!(2024-01-01 00:05:00).assume_utc();
        for ((s, i), v) in portfolio.positions(&event_time).iter() {
            info!("{}: {}: {}", s, i, v);
        }
        // assert_eq!(position.avg_price, Price::from(0.));
        // assert_eq!(position.quantity, Quantity::from(0.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(2000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(0.));
    }
}
