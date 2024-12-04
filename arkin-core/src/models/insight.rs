use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{Event, EventType, EventTypeOf, FeatureId};

use super::{Instrument, Pipeline};

#[derive(Debug, Clone, TypedBuilder)]

pub struct Insight {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub pipeline: Arc<Pipeline>,
    pub instrument: Option<Arc<Instrument>>,
    pub feature_id: FeatureId,
    pub value: Decimal,
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
