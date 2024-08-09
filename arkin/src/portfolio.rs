use std::{collections::HashMap, sync::Arc};

use time::OffsetDateTime;

use crate::{
    models::{Fill, Instrument, Notional, Position, Price, Quantity},
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

    pub fn absolut_exposure(&self, event_time: &OffsetDateTime) -> Notional {
        self.positions(event_time)
            .values()
            .map(|p| p.avg_price * p.quantity.abs())
            .sum()
    }

    pub fn total_exposure(&self, event_time: &OffsetDateTime) -> Notional {
        self.positions(event_time).values().map(|p| p.avg_price * p.quantity).sum()
    }

    pub fn position(&self, strategy_id: &StrategyId, instrument: &Instrument, timestamp: &OffsetDateTime) -> Position {
        let fills = self
            .state
            .list_events_since_beginning::<Fill>(instrument, timestamp)
            .into_iter()
            .filter(|fill| fill.strategy_id == *strategy_id)
            .collect::<Vec<_>>();

        fills
            .into_iter()
            .fold(None, |position: Option<Position>, fill| {
                let mut position = position.unwrap_or_else(|| Position::from_fill(&fill));
                position.update(&fill);
                Some(position)
            })
            .unwrap_or_else(|| {
                Position::new(
                    strategy_id.clone(),
                    instrument.clone(),
                    *timestamp,
                    Price::from(0.),
                    Quantity::from(0.),
                )
            })
    }

    pub fn positions(&self, event_time: &OffsetDateTime) -> HashMap<Instrument, Position> {
        let mut positions = HashMap::new();
        for instrument in self.state.list_instruments() {
            let position = self.position(&instrument, event_time);
            positions.insert(instrument.clone(), position);
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

        let instrument = test_utils::test_perp_instrument();
        let state = test_utils::TestStateBuilder::default().add_fills(&instrument).build();
        let portfolio = Portfolio::new(state, Notional::from(2000.));

        let mut event_time = datetime!(2024-01-01 00:00:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(0.));
        assert_eq!(position.quantity, Quantity::from(0.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(2000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(0.));

        event_time = datetime!(2024-01-01 00:01:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(80.));
        assert_eq!(position.quantity, Quantity::from(10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(1200.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(800.));

        event_time = datetime!(2024-01-01 00:02:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(100.));
        assert_eq!(position.quantity, Quantity::from(20.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(0.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(2000.));

        event_time = datetime!(2024-01-01 00:03:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(100.));
        assert_eq!(position.quantity, Quantity::from(10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(1000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(1000.));

        event_time = datetime!(2024-01-01 00:04:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(100.));
        assert_eq!(position.quantity, Quantity::from(-10.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(3000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(-1000.));

        event_time = datetime!(2024-01-01 00:05:00).assume_utc();
        let position = portfolio.position(&instrument, &event_time);
        info!("{}", position);
        assert_eq!(position.avg_price, Price::from(0.));
        assert_eq!(position.quantity, Quantity::from(0.));
        assert_eq!(portfolio.buying_power(&event_time), Notional::from(2000.));
        assert_eq!(portfolio.total_exposure(&event_time), Notional::from(0.));
    }
}
