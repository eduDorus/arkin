use std::fmt;

use strum::Display;
use time::OffsetDateTime;

use crate::{
    constants::TIMESTAMP_FORMAT,
    events::{Event, EventType, EventTypeOf},
    types::{Price, Quantity},
};

use super::{Account, ExecutionOrder, Instrument, Strategy, VenueOrder};

#[derive(Display, Clone, Copy, PartialEq, Eq)]
pub enum FillSide {
    Buy,
    Sell,
}

#[derive(Clone)]
pub struct Fill {
    pub id: u32,
    pub account: Account,
    pub instrument: Instrument,
    pub strategy: Strategy,
    pub execution_order: ExecutionOrder,
    pub venue_order: VenueOrder,
    pub side: FillSide,
    pub price: Price,
    pub quantity: Quantity,
    pub created_at: OffsetDateTime,
}

impl Fill {
    pub fn new(
        id: u32,
        account: Account,
        instrument: Instrument,
        strategy: Strategy,
        execution_order: ExecutionOrder,
        venue_order: VenueOrder,
        side: FillSide,
        price: Price,
        quantity: Quantity,
        created_at: OffsetDateTime,
    ) -> Self {
        Fill {
            id,
            account,
            instrument,
            strategy,
            execution_order,
            venue_order,
            side,
            price,
            quantity,
            created_at,
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
            "{} {} {} {} side: {} price: {} quantity: {}",
            self.created_at.format(TIMESTAMP_FORMAT).unwrap(),
            self.account,
            self.instrument,
            self.strategy,
            self.side,
            self.price,
            self.quantity,
        )
    }
}
