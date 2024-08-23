use std::fmt;

use time::OffsetDateTime;

use crate::{
    events::{EventType, EventTypeOf},
    Event, Price, Quantity,
};

use super::Instrument;

#[derive(Clone)]
pub struct Trade {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub trade_id: u64,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl Trade {
    pub fn new(
        received_time: OffsetDateTime,
        event_time: OffsetDateTime,
        instrument: Instrument,
        trade_id: u64,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            received_time,
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

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
    }
}
