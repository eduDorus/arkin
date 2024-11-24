use std::{fmt, sync::Arc};

use derive_builder::Builder;
use time::OffsetDateTime;

use crate::{constants, Event, StrategyId, UpdateEventType, Weight};

use super::Instrument;

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Signal {
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub strateg_id: StrategyId,
    pub weight: Weight,
}

impl Event for Signal {
    fn event_type() -> UpdateEventType {
        UpdateEventType::Signal
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
