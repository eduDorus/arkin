use std::fmt;

use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{
    events::{Event, EventType, EventTypeOf},
    types::{Notional, Price, Quantity, StrategyId, Weight},
};

use super::Instrument;

#[derive(Clone)]
pub struct Allocation {
    pub event_time: OffsetDateTime,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub current_allocation: Weight,
    pub current_price: Price,
    pub current_quantity: Quantity,
    pub target_allocation: Weight,
    pub target_price: Price,
    pub target_quantity: Quantity,
    pub value_at_risk: Notional,
    pub expected_shortfall: Notional,
    pub beta: Decimal,
}

impl EventTypeOf for Allocation {
    fn event_type() -> EventType {
        EventType::Allocation
    }
}

impl TryFrom<Event> for Allocation {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Allocation(allocation) = event {
            Ok(allocation)
        } else {
            Err(())
        }
    }
}

impl From<Allocation> for Event {
    fn from(allocation: Allocation) -> Self {
        Event::Allocation(allocation)
    }
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.current_allocation, self.target_allocation,
        )
    }
}
