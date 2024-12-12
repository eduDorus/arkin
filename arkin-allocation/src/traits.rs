use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use tokio_util::sync::CancellationToken;

use arkin_core::prelude::*;

use crate::AllocationOptimError;

#[automock]
#[async_trait]
pub trait AllocationOptim: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), AllocationOptimError>;

    async fn optimize(&self, tick: Arc<InsightTick>) -> Result<Vec<Arc<ExecutionOrder>>, AllocationOptimError>;
}
