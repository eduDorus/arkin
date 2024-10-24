use std::{fmt, sync::Arc};

use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    events::{EventType, EventTypeOf},
    Event, Weight,
};

use super::{Instrument, Strategy};

#[derive(Clone)]
pub struct Signal {
    pub id: Uuid,
    pub instrument: Arc<Instrument>,
    pub strategy: Strategy,
    pub weight: Weight,
    pub created_at: OffsetDateTime,
}

impl Signal {
    pub fn new(instrument: Arc<Instrument>, strategy: Strategy, weight: Weight, created_at: OffsetDateTime) -> Self {
        Signal {
            id: Uuid::new_v4(),
            instrument,
            strategy,
            weight,
            created_at,
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
            "{} {} {} {}",
            self.created_at, self.strategy.name, self.instrument.symbol, self.weight
        )
    }
}
