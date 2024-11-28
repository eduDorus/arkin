use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{
    types::{Commission, MarketValue},
    Event, EventType, EventTypeOf, Notional, Price, Quantity,
};

use super::{ExecutionOrderId, Instrument, MarketSide, VenueOrderFill};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct ExecutionOrderFill {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl ExecutionOrderFill {
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

impl EventTypeOf for ExecutionOrderFill {
    fn event_type() -> EventType {
        EventType::ExecutionOrderFill
    }
}

impl From<ExecutionOrderFill> for Event {
    fn from(update: ExecutionOrderFill) -> Self {
        Event::ExecutionOrderFill(update)
    }
}

impl From<VenueOrderFill> for ExecutionOrderFill {
    fn from(fill: VenueOrderFill) -> Self {
        ExecutionOrderFill {
            event_time: fill.event_time,
            id: fill.execution_order_id,
            instrument: fill.instrument,
            side: fill.side,
            price: fill.price,
            quantity: fill.quantity,
            commission: fill.commission,
        }
    }
}

impl fmt::Display for ExecutionOrderFill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} price={} quantity={} commission={}",
            self.instrument, self.side, self.price, self.quantity, self.commission
        )
    }
}
