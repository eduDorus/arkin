use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use arkin_core::prelude::*;

use crate::AllocationOptimError;

#[automock]
#[async_trait]
pub trait AllocationOptim: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), AllocationOptimError>;

    async fn list_signals(&self) -> Result<Vec<Arc<Signal>>, AllocationOptimError>;

    async fn new_signal(&self, signal: Arc<Signal>) -> Result<(), AllocationOptimError>;
    async fn new_signals(&self, signals: Vec<Arc<Signal>>) -> Result<(), AllocationOptimError>;
    async fn optimize(&self, event_time: OffsetDateTime) -> Result<Vec<Arc<ExecutionOrder>>, AllocationOptimError>;
}
