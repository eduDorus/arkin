use std::sync::Arc;

use async_trait::async_trait;
use derive_builder::Builder;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Executor, OrderManager, OrderManagerError};

#[derive(Debug, Builder)]
pub struct SingleExecutorOrderManager {
    executor: Arc<dyn Executor>,
}

#[async_trait]
impl OrderManager for SingleExecutorOrderManager {
    #[instrument(skip(self))]
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        self.executor.start(task_tracker.clone(), shutdown.clone()).await?;
        info!("Order manager started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), OrderManagerError> {
        info!("Cleaning up order manager...");
        self.executor.cleanup().await?;
        info!("Order manager cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn new_order(&self, _order: ExecutionOrder) -> Result<(), OrderManagerError> {
        // self.executor.place_order(order).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_order(&self, _order_id: u64) -> Result<(), OrderManagerError> {
        // self.executor.cancel_order(order_id).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError> {
        self.executor.cancel_all_orders().await?;
        Ok(())
    }
}
