use std::fmt;

use time::OffsetDateTime;

use crate::{
    events::{EventType, EventTypeOf},
    models::Insight,
    Event, Price, Quantity,
};

use super::Instrument;

#[derive(Clone)]
pub struct Trade {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub trade_id: u64,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl Trade {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        trade_id: u64,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            trade_id,
            price,
            quantity,
        }
    }
}

impl EventTypeOf for Trade {
    fn event_type() -> EventType {
        EventType::Trade
    }
}

impl TryFrom<Event> for Trade {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Trade(trade) = event {
            Ok(trade)
        } else {
            Err(())
        }
    }
}

impl From<Trade> for Event {
    fn from(v: Trade) -> Self {
        Event::Trade(v)
    }
}

impl From<Trade> for Vec<Insight> {
    fn from(v: Trade) -> Self {
        vec![
            Insight::new("trade_price".into(), v.instrument.clone(), v.event_time, v.price),
            Insight::new("trade_quantity".into(), v.instrument.clone(), v.event_time, v.quantity),
        ]
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
    }
}
