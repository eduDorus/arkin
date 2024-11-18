use async_trait::async_trait;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use arkin_core::prelude::*;

use crate::AllocationOptimError;

#[async_trait]
pub trait AllocationOptim: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), AllocationOptimError>;
    async fn cleanup(&self) -> Result<(), AllocationOptimError>;
    async fn new_signal(&self, signal: Signal) -> Result<(), AllocationOptimError>;
    async fn new_signals(&self, signals: Vec<Signal>) -> Result<(), AllocationOptimError>;
    async fn optimize(&self) -> Result<Vec<ExecutionOrder>, AllocationOptimError>;
}
