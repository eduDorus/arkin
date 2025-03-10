use std::{fmt, sync::Arc};

use strum::Display;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{Event, EventType, EventTypeOf, FeatureId};

use super::{Instrument, Pipeline};

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum InsightType {
    Raw,
    Ohlcv,
    Price,
    MovingAverage,
    Continuous,
    Categorical,
    Normalized,
    Prediction,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Insight {
    pub event_time: OffsetDateTime,
    #[builder(default)]
    pub pipeline: Option<Arc<Pipeline>>,
    #[builder(default)]
    pub instrument: Option<Arc<Instrument>>,
    pub feature_id: FeatureId,
    pub value: f64,
    pub insight_type: InsightType,
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
