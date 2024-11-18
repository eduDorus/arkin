use async_trait::async_trait;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::IngestorError;

#[async_trait]
pub trait Ingestor: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), IngestorError>;
    async fn cleanup(&self) -> Result<(), IngestorError>;
}
