use std::sync::Arc;

use time::macros::datetime;

use crate::{
    models::{Allocation, Event, Fill, Instrument, Notional, Price, Quantity, Tick, Venue},
    state::StateManager,
};
pub fn test_perp_instrument() -> Instrument {
    // Create an instrument
    Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into())
}

pub fn test_multi_perp_instrument() -> Vec<Instrument> {
    // Create an instrument
    vec![
        Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
        Instrument::perpetual(Venue::Binance, "ETH".into(), "USDT".into()),
    ]
}

#[derive(Default)]
pub struct TestStateBuilder {
    state: Arc<StateManager>,
}

impl TestStateBuilder {
    pub fn add_fills(self, instrument: &Instrument) -> Self {
        // Create a couple of fills
        self.state.add_event(Event::Fill(Fill {
            event_time: datetime!(2024-01-01 00:00:00).assume_utc(),
            instrument: instrument.clone(),
            order_id: 0,
            strategy_id: "test_strategy".into(),
            price: Price::from(80.),
            quantity: Quantity::from(10.),
            commission: Notional::from(1.5),
        }));
        self.state.add_event(Event::Fill(Fill {
            event_time: datetime!(2024-01-01 00:01:00).assume_utc(),
            instrument: instrument.clone(),
            order_id: 1,
            strategy_id: "test_strategy".into(),
            price: Price::from(120.),
            quantity: Quantity::from(10.),
            commission: Notional::from(1.0),
        }));
        self.state.add_event(Event::Fill(Fill {
            event_time: datetime!(2024-01-01 00:02:00).assume_utc(),
            instrument: instrument.clone(),
            order_id: 2,
            strategy_id: "test_strategy".into(),
            price: Price::from(100.),
            quantity: Quantity::from(-10.),
            commission: Notional::from(1.5),
        }));
        self.state.add_event(Event::Fill(Fill {
            event_time: datetime!(2024-01-01 00:03:00).assume_utc(),
            instrument: instrument.clone(),
            order_id: 3,
            strategy_id: "test_strategy".into(),
            price: Price::from(100.),
            quantity: Quantity::from(-20.),
            commission: Notional::from(2.),
        }));
        self.state.add_event(Event::Fill(Fill {
            event_time: datetime!(2024-01-01 00:04:00).assume_utc(),
            instrument: instrument.clone(),
            order_id: 3,
            strategy_id: "test_strategy".into(),
            price: Price::from(50.),
            quantity: Quantity::from(10.),
            commission: Notional::from(2.),
        }));

        self
    }

    pub fn add_ticks(self, instrument: &Instrument) -> Self {
        // Create a couple of trades
        self.state.add_event(Event::Tick(Tick::new(
            datetime!(2024-01-01 00:00:00).assume_utc(),
            instrument.clone(),
            0,
            Price::from(101.),
            Quantity::from(10.),
            Price::from(100.),
            Quantity::from(10.),
        )));

        self.state.add_event(Event::Tick(Tick::new(
            datetime!(2024-01-01 00:01:00).assume_utc(),
            instrument.clone(),
            1,
            Price::from(102.),
            Quantity::from(10.),
            Price::from(101.),
            Quantity::from(10.),
        )));

        self.state.add_event(Event::Tick(Tick::new(
            datetime!(2024-01-01 00:02:00).assume_utc(),
            instrument.clone(),
            2,
            Price::from(103.),
            Quantity::from(10.),
            Price::from(102.),
            Quantity::from(10.),
        )));

        self.state.add_event(Event::Tick(Tick::new(
            datetime!(2024-01-01 00:03:00).assume_utc(),
            instrument.clone(),
            3,
            Price::from(104.),
            Quantity::from(10.),
            Price::from(101.),
            Quantity::from(10.),
        )));

        self
    }

    pub fn build(self) -> Arc<StateManager> {
        self.state
    }
}

pub fn allocations(instrument: &Instrument) -> Vec<Allocation> {
    vec![Allocation::new(
        datetime!(2024-01-01 00:00:00).assume_utc(),
        instrument.clone(),
        "test_strategy".into(),
        Notional::from(1000.),
    )]
}
