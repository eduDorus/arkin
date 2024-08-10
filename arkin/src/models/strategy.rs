use std::fmt;
use time::OffsetDateTime;

use crate::strategies::StrategyId;

use super::{Event, EventType, EventTypeOf, Instrument, Weight};

#[derive(Clone)]
pub struct Signal {
    pub event_time: OffsetDateTime,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub signal: Weight,
}

impl Signal {
    pub fn new(event_time: OffsetDateTime, instrument: Instrument, strategy_id: StrategyId, signal: Weight) -> Self {
        Signal {
            event_time,
            instrument,
            strategy_id,
            signal,
        }
    }
}

impl EventTypeOf for Signal {
    fn event_type() -> EventType {
        EventType::Signal
    }
}

impl TryFrom<Event> for Signal {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Signal(tick) = event {
            Ok(tick)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.signal
        )
    }
}
