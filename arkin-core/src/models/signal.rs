use std::{fmt, sync::Arc};

use derive_builder::Builder;
use time::OffsetDateTime;

use crate::{
    constants,
    events::{EventType, EventTypeOf},
    Event, StrategyId, Weight,
};

use super::Instrument;

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Signal {
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub strateg_id: StrategyId,
    pub weight: Weight,
}

impl EventTypeOf for Signal {
    fn event_type() -> EventType {
        EventType::Signal
    }
}

impl TryFrom<Event> for Signal {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Signal(signal) = event {
            Ok(signal)
        } else {
            Err(())
        }
    }
}

impl From<Signal> for Event {
    fn from(signal: Signal) -> Self {
        Event::Signal(signal)
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Signal: ts={} inst={} strategy={} weight={}",
            self.event_time
                .format(constants::TIMESTAMP_FORMAT)
                .expect("Failed to format timestamp"),
            self.instrument.symbol,
            self.strateg_id,
            self.weight
        )
    }
}
