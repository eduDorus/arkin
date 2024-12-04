use std::{sync::Arc, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use arkin_core::prelude::*;

use crate::InsightsError;

#[async_trait]
pub trait Insights: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), InsightsError>;

    async fn insert(&self, insight: Arc<Insight>) -> Result<(), InsightsError>;
    async fn insert_batch(&self, insights: &[Arc<Insight>]) -> Result<(), InsightsError>;

    /// Remove all insights with event_time <= provided event_time
    async fn remove(&self, event_time: OffsetDateTime) -> Result<(), InsightsError>;

    async fn load(
        &self,
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        lookback: Duration,
    ) -> Result<(), InsightsError>;

    async fn process(
        &self,
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        publish: bool,
    ) -> Result<Vec<Arc<Insight>>, InsightsError>;
}

pub trait Computation: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>>;
}
