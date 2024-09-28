use std::fmt;

use time::OffsetDateTime;

use crate::{
    events::{EventType, EventTypeOf},
    models::Insight,
    Event, Price, Quantity,
};

use super::{Instrument, MarketSide};

#[derive(Clone)]
pub struct Trade {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub trade_id: u64,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
}

impl Trade {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        trade_id: u64,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            trade_id,
            side,
            price,
            quantity,
        }
    }

    pub fn to_insights(&self) -> Vec<Insight> {
        vec![
            Insight::new("trade_price".into(), self.instrument.clone(), self.event_time, self.price),
            Insight::new("trade_quantity".into(), self.instrument.clone(), self.event_time, self.quantity),
        ]
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

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
    }
}
