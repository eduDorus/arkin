use std::sync::Arc;

use async_trait::async_trait;

use arkin_core::prelude::*;

use crate::AllocationOptimError;

#[async_trait]
pub trait AllocationOptim: std::fmt::Debug + Send + Sync {
    async fn optimize(&self, signal: Arc<Signal>) -> Result<Vec<Arc<ExecutionOrder>>, AllocationOptimError>;
}

#[async_trait]
pub trait AllocationService: RunnableService + AllocationOptim {}
