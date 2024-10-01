use std::fmt;

use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{constants::TIMESTAMP_FORMAT, FeatureId};

use super::Instrument;

#[derive(Clone)]
pub struct Insight {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub feature_id: FeatureId,
    pub value: Decimal,
}

impl Insight {
    pub fn new(event_time: OffsetDateTime, instrument: Instrument, feature_id: FeatureId, value: Decimal) -> Self {
        Insight {
            event_time,
            instrument,
            feature_id,
            value,
        }
    }

    pub fn id(&self) -> &FeatureId {
        &self.feature_id
    }

    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    pub fn event_time(&self) -> &OffsetDateTime {
        &self.event_time
    }

    pub fn value(&self) -> &Decimal {
        &self.value
    }
}

impl fmt::Display for Insight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format time");
        write!(f, "{} {} {} {}", event_time, self.instrument, self.feature_id, self.value)
    }
}
