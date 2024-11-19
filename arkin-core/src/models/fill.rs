use std::{fmt, sync::Arc};

use derive_builder::Builder;
use strum::Display;
use uuid::Uuid;

use crate::{
    events::{Event, EventType, EventTypeOf},
    types::{Commission, Price, Quantity},
};

use super::{Instrument, MarketSide, VenueOrderId};

#[derive(Display, Clone, Copy, PartialEq, Eq)]
pub enum FillSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Fill {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub venue_order_id: VenueOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
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
            "instrument: {} side: {} price: {} quantity: {}",
            self.instrument, self.side, self.price, self.quantity,
        )
    }
}
