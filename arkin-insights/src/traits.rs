use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use arkin_core::prelude::*;

use crate::InsightsError;

#[async_trait]
pub trait Insights: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), InsightsError>;
    async fn cleanup(&self) -> Result<(), InsightsError>;

    async fn insert(&self, insight: Insight) -> Result<(), InsightsError>;
    async fn insert_batch(&self, insights: Vec<Insight>) -> Result<(), InsightsError>;

    async fn load(
        &self,
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
        frequency: Duration,
    ) -> Result<(), InsightsError>;

    async fn process(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<(), InsightsError>;
}
