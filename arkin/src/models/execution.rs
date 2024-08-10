use crate::constants::TIMESTAMP_FORMAT;

use super::{Event, EventType, EventTypeOf, Instrument, Notional, Price, Quantity};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Fill {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub execution_id: u64,
    pub order_id: u64,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Notional,
}

impl Fill {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        execution_id: u64,
        order_id: u64,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Self {
        Self {
            event_time,
            instrument,
            execution_id,
            order_id,
            price,
            quantity,
            commission,
        }
    }

    pub fn notional(&self) -> Notional {
        self.price * self.quantity.abs()
    }
}

impl EventTypeOf for Fill {
    fn event_type() -> EventType {
        EventType::Fill
    }
}

impl TryFrom<Event> for Fill {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Fill(fill) = event {
            Ok(fill)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FILL {} {} avg price: {} quantity: {} commission: {}",
            self.event_time.format(TIMESTAMP_FORMAT).unwrap(),
            self.instrument,
            self.price,
            self.quantity,
            self.commission
        )
    }
}
