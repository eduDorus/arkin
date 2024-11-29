use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::{
    types::{Commission, MarketValue},
    Event, EventType, EventTypeOf, Notional, Price, Quantity,
};

use super::{Instance, Instrument, MarketSide, VenueOrder};

#[derive(Debug, Clone, Builder, FromRow)]
#[builder(setter(into))]
pub struct VenueOrderFill {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub instance: Arc<Instance>,
    pub venue_order: Arc<VenueOrder>,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl VenueOrderFill {
    /// The total value of your current position based on the latest market prices.
    pub fn market_value(&self) -> MarketValue {
        self.price * self.quantity_with_side() * self.instrument.contract_size * Decimal::NEGATIVE_ONE
    }

    /// The total value of the underlying asset that a financial derivative represents. It provides a measure of the total exposure.
    pub fn notional_value(&self) -> Notional {
        self.price * self.quantity * self.instrument.contract_size
    }

    pub fn total_cost(&self) -> Decimal {
        self.market_value() - self.commission
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
