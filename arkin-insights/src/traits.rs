use std::sync::Arc;

use async_trait::async_trait;
use time::UtcDateTime;

use arkin_core::prelude::*;

#[async_trait]
pub trait Feature: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instrument: &Arc<Instrument>, event_time: UtcDateTime) -> Option<Vec<Insight>>;
    async fn async_calculate(&self, instrument: &Arc<Instrument>, event_time: UtcDateTime) -> Option<Vec<Insight>>;
}
