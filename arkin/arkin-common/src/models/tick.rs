use std::fmt;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::OffsetDateTime;

use crate::{
    events::{EventType, EventTypeOf},
    Event, Price, Quantity,
};

use super::{Feature, Instrument};

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

    pub fn spread(&self) -> Decimal {
        self.ask_price - self.bid_price
    }

    pub fn mid_price(&self) -> Price {
        (self.bid_price + self.ask_price) / dec!(2)
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

impl From<Tick> for Vec<Feature> {
    fn from(value: Tick) -> Self {
        vec![
            Feature::new("bid_price".into(), value.instrument.clone(), value.event_time, value.bid_price),
            Feature::new("ask_price".into(), value.instrument.clone(), value.event_time, value.ask_price),
        ]
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
