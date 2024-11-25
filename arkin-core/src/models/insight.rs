use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{constants::TIMESTAMP_FORMAT, Event, FeatureId, UpdateEvent, UpdateEventType};

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

impl Event for Insight {
    fn event_type() -> UpdateEventType {
        UpdateEventType::Insight
    }
}

impl From<Insight> for UpdateEvent {
    fn from(insight: Insight) -> Self {
        UpdateEvent::Insight(insight)
    }
}

impl fmt::Display for Insight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format time");
        write!(
            f,
            "{} {:?} {} {}",
            event_time,
            self.instrument.as_ref().map(|i| i.symbol.clone()),
            self.feature_id,
            self.value
        )
    }
}
