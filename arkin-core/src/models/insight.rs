use std::{fmt, sync::Arc, time::Duration};

use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::FeatureId;

use super::{Instrument, Pipeline};

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsTick {
    pub event_time: UtcDateTime,
    pub frequency: Duration,
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
    pub event_time: UtcDateTime,
    #[builder(default)]
    pub pipeline: Option<Arc<Pipeline>>,
    pub instrument: Arc<Instrument>,
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
            self.instrument, self.feature_id, self.value
        )
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsUpdate {
    pub event_time: UtcDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub insights: Vec<Arc<Insight>>,
}
