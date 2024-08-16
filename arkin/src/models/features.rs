use std::fmt;

use time::OffsetDateTime;

use crate::constants::TIMESTAMP_FORMAT;

use super::{FeatureId, Instrument};

#[derive(Clone)]
pub struct FeatureEvent {
    pub id: FeatureId,
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub value: f64,
}

impl FeatureEvent {
    pub fn new(id: FeatureId, instrument: Instrument, event_time: OffsetDateTime, value: f64) -> Self {
        FeatureEvent {
            id,
            instrument,
            event_time,
            value,
        }
    }
}

impl fmt::Display for FeatureEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format time");
        write!(f, "{} {} {} {}", event_time, self.instrument, self.id, self.value)
    }
}
