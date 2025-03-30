use std::{fmt, sync::Arc, time::Duration};

use strum::Display;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{Event, FeatureId};

use super::{Instrument, Pipeline};

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub frequency: Duration,
}

impl From<InsightsTick> for Event {
    fn from(tick: InsightsTick) -> Self {
        Event::InsightsTick(Arc::new(tick))
    }
}

impl From<Arc<InsightsTick>> for Event {
    fn from(tick: Arc<InsightsTick>) -> Self {
        Event::InsightsTick(tick)
    }
}

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

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsUpdate {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub insights: Vec<Arc<Insight>>,
}

impl From<Arc<InsightsUpdate>> for Event {
    fn from(tick: Arc<InsightsUpdate>) -> Self {
        Event::InsightsUpdate(tick)
    }
}
