use std::fmt;

use time::OffsetDateTime;

use crate::{
    constants::TIMESTAMP_FORMAT,
    events::{Event, EventType, EventTypeOf},
    types::{Notional, Price, Quantity, StrategyId},
};

use super::{Instrument, Side, Venue};

#[derive(Clone)]
pub struct Fill {
    pub event_time: OffsetDateTime,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub order_id: u64,
    pub venue_order_id: u64,
    pub venue: Venue,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Notional,
}

impl Fill {
    pub fn new(
        event_time: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        order_id: u64,
        venue_order_id: u64,
        venue: Venue,
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Self {
        Self {
            event_time,
            strategy_id,
            instrument,
            order_id,
            venue_order_id,
            venue,
            side,
            price,
            quantity,
            commission,
        }
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
            "FILL {} {} side: {} avg price: {} quantity: {} commission: {}",
            self.event_time.format(TIMESTAMP_FORMAT).unwrap(),
            self.instrument,
            self.side,
            self.price.round_dp(2),
            self.quantity.round_dp(4),
            self.commission.round_dp(2)
        )
    }
}
