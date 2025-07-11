use std::sync::Arc;

use time::UtcDateTime;

use arkin_core::prelude::*;

pub trait Feature: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instrument: &Arc<Instrument>, event_time: UtcDateTime) -> Option<Vec<Arc<Insight>>>;
}
