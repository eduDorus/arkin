use std::{fmt, sync::Arc};

use derive_builder::Builder;
use strum::Display;
use uuid::Uuid;

use crate::{
    types::{Commission, Price, Quantity},
    Event, UpdateEventType,
};

use super::{ExecutionOrderId, Instrument, MarketSide, VenueOrderId};

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
    pub execution_order_id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl Event for Fill {
    fn event_type() -> UpdateEventType {
        UpdateEventType::Fill
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
