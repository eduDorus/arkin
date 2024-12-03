use std::{fmt, sync::Arc};

use derive_builder::Builder;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{EventType, EventTypeOf, Weight};

use super::{Instrument, Strategy};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Builder)]
#[builder(setter(into))]
pub struct Signal {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub strategy: Arc<Strategy>,
    pub weight: Weight,
}

impl EventTypeOf for Signal {
    fn event_type() -> EventType {
        EventType::Signal
    }
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
