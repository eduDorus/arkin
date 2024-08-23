use std::fmt;

use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{constants::TIMESTAMP_FORMAT, FeatureId};

use super::Instrument;

#[derive(Clone)]
pub struct Feature {
    pub id: FeatureId,
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub value: Decimal,
}

impl Feature {
    pub fn new(id: FeatureId, instrument: Instrument, event_time: OffsetDateTime, value: Decimal) -> Self {
        Feature {
            id,
            instrument,
            event_time,
            value,
        }
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format time");
        write!(f, "{} {} {} {}", event_time, self.instrument, self.id, self.value)
    }
}
