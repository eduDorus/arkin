use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{Event, EventType, EventTypeOf, FeatureId};

use super::Instrument;

#[derive(Debug, Clone)]
pub struct Insight {
    pub event_time: OffsetDateTime,
    pub instrument: Option<Arc<Instrument>>,
    pub feature_id: FeatureId,
    pub value: Decimal,
}

impl Insight {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        value: Decimal,
    ) -> Self {
        Insight {
            event_time,
            instrument,
            feature_id,
            value,
        }
    }

    pub fn new_general(event_time: OffsetDateTime, feature_id: FeatureId, value: Decimal) -> Self {
        Self {
            event_time,
            instrument: None,
            feature_id,
            value,
        }
    }
}

impl EventTypeOf for Insight {
    fn event_type() -> EventType {
        EventType::Insight
    }
}

impl From<Insight> for Event {
    fn from(insight: Insight) -> Self {
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
