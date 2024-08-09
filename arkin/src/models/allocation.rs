use super::{Event, EventType, EventTypeOf, Instrument, Notional};
use crate::strategies::StrategyId;
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Allocation {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub strategy_id: StrategyId,
    pub notional: Notional,
}

impl Allocation {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        strategy_id: StrategyId,
        notional: Notional,
    ) -> Self {
        Self {
            event_time,
            instrument,
            strategy_id,
            notional,
        }
    }
}

impl EventTypeOf for Allocation {
    fn event_type() -> EventType {
        EventType::Tick
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

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.notional
        )
    }
}
