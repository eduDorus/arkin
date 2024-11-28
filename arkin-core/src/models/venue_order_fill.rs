use std::{fmt, sync::Arc};

use derive_builder::Builder;
use time::OffsetDateTime;

use crate::{
    types::{Commission, MarketValue},
    Event, EventType, EventTypeOf, Notional, Price, Quantity,
};

use super::{ExecutionOrderId, Instrument, MarketSide, VenueOrderId};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct VenueOrderFill {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: VenueOrderId,
    pub execution_order_id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl VenueOrderFill {
    /// The total value of your current position based on the latest market prices.
    pub fn market_value(&self) -> MarketValue {
        self.price * self.quantity_with_side() * self.instrument.contract_size
    }

    /// The total value of the underlying asset that a financial derivative represents. It provides a measure of the total exposure.
    pub fn notional_value(&self) -> Notional {
        self.price * self.quantity * self.instrument.contract_size
    }

    pub fn quantity_with_side(&self) -> Quantity {
        match self.side {
            MarketSide::Buy => self.quantity,
            MarketSide::Sell => -self.quantity,
        }
    }
}

impl EventTypeOf for VenueOrderFill {
    fn event_type() -> EventType {
        EventType::VenueOrderFill
    }
}

impl From<VenueOrderFill> for Event {
    fn from(update: VenueOrderFill) -> Self {
        Event::VenueOrderFill(update)
    }
}

impl fmt::Display for VenueOrderFill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} price={} quantity={} commission={}",
            self.instrument, self.side, self.price, self.quantity, self.commission
        )
    }
}
