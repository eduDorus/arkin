use std::{collections::HashMap, sync::Arc};

use time::OffsetDateTime;

use crate::{
    models::{Event, EventType, Instrument, Notional, Position, Price, Quantity},
    state::State,
};

pub struct Portfolio {
    state: Arc<State>,
    capital: Notional,
}

impl Portfolio {
    pub fn new(state: Arc<State>, capital: Notional) -> Self {
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

    pub fn position(&self, instrument: &Instrument, event_time: &OffsetDateTime) -> Position {
        self.state
            .list_events_since_beginning(instrument, &EventType::Fill, event_time)
            .into_iter()
            .fold(Position::new(instrument.clone(), *event_time), |mut acc, x| {
                if let Event::Fill(f) = x {
                    let price_sum = acc.avg_price * acc.quantity + f.price * f.quantity.abs();
                    let quantity_sum = acc.quantity + f.quantity.abs();
                    let quantity_real = acc.quantity + f.quantity;
                    if quantity_real == Quantity::from(0.) {
                        acc.avg_price = Price::from(0.);
                        acc.quantity = quantity_real;
                    } else {
                        acc.avg_price = price_sum / quantity_sum;
                        acc.quantity = quantity_real;
                    }
                }
                acc
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
