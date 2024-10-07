use std::fmt;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::OffsetDateTime;

use crate::{
    events::{EventType, EventTypeOf},
    Event, Price, Quantity,
};

use super::{Insight, Instrument};

#[derive(Clone)]
pub struct Tick {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub tick_id: u64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl Tick {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        tick_id: u64,
        bid_price: Price,
        bid_quantity: Quantity,
        ask_price: Price,
        ask_quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            tick_id,
            bid_price,
            bid_quantity,
            ask_price,
            ask_quantity,
        }
    }

    pub fn to_insights(&self) -> Vec<Insight> {
        vec![
            Insight::new(
                self.event_time,
                Some(self.instrument.clone()),
                "bid_price".into(),
                self.bid_price,
            ),
            Insight::new(
                self.event_time,
                Some(self.instrument.clone()),
                "ask_price".into(),
                self.ask_price,
            ),
            Insight::new(
                self.event_time,
                Some(self.instrument.clone()),
                "mid_price".into(),
                self.mid_price(),
            ),
            Insight::new(self.event_time, Some(self.instrument.clone()), "spread".into(), self.spread()),
        ]
    }

    pub fn spread(&self) -> Decimal {
        self.ask_price - self.bid_price
    }

    pub fn mid_price(&self) -> Price {
        (self.bid_price + self.ask_price) / dec!(2)
    }

    pub fn bid_price(&self) -> Price {
        self.bid_price
    }

    pub fn ask_price(&self) -> Price {
        self.ask_price
    }
}

impl EventTypeOf for Tick {
    fn event_type() -> EventType {
        EventType::Tick
    }
}

impl TryFrom<Event> for Tick {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Tick(tick) = event {
            Ok(tick)
        } else {
            Err(())
        }
    }
}

impl From<Tick> for Event {
    fn from(value: Tick) -> Self {
        Event::Tick(value)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} bid: {} {} ask: {} {}",
            self.instrument, self.event_time, self.bid_price, self.bid_quantity, self.ask_price, self.ask_quantity
        )
    }
}