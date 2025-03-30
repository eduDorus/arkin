use std::{fmt, sync::Arc};

use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{Event, Weight};

use super::{Instrument, Strategy};

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
pub struct Signal {
    pub event_time: OffsetDateTime,
    pub strategy: Arc<Strategy>,
    pub instrument: Arc<Instrument>,
    pub weight: Weight,
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} strategy={} weight={}",
            self.instrument.symbol, self.strategy.name, self.weight
        )
    }
}

impl From<Signal> for Event {
    fn from(signal: Signal) -> Self {
        Event::SignalUpdate(Arc::new(signal))
    }
}

impl From<Arc<Signal>> for Event {
    fn from(signal: Arc<Signal>) -> Self {
        Event::SignalUpdate(signal)
    }
}
