use std::sync::Arc;

use async_trait::async_trait;
use time::UtcDateTime;

use arkin_core::prelude::*;

use crate::FeatureState;

#[async_trait]
pub trait Feature: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(
        &self,
        state: &FeatureState,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>>;
}
