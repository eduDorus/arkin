use std::{fmt, sync::Arc};

use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{Event, EventType, EventTypeOf, FeatureId};

use super::{Instrument, Pipeline};

#[derive(Debug, Clone, TypedBuilder)]

pub struct Insight {
    pub event_time: OffsetDateTime,
    pub pipeline: Arc<Pipeline>,
    pub instrument: Option<Arc<Instrument>>,
    pub feature_id: FeatureId,
    pub value: f64,
    #[builder(default = false)]
    pub persist: bool,
}

impl EventTypeOf for Insight {
    fn event_type() -> EventType {
        EventType::Insight
    }
}

impl From<Arc<Insight>> for Event {
    fn from(insight: Arc<Insight>) -> Self {
        Event::Insight(insight)
    }
}

impl fmt::Display for Insight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "instrument={} feature={} value={}",
            self.instrument
                .as_ref()
                .map(|instrument| instrument.symbol.as_str())
                .unwrap_or("global value"),
            self.feature_id,
            self.value
        )
    }
}
