use std::sync::Arc;

use async_trait::async_trait;

use arkin_core::prelude::*;

#[async_trait]
pub trait AllocationOptim: Send + Sync {
    async fn optimize(&self, signal: Arc<Signal>);
}

#[async_trait]
pub trait AllocationService: RunnableService + AllocationOptim {}
