use async_trait::async_trait;

use arkin_core::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{ExecutorError, OrderManagerError};

#[async_trait]
pub trait OrderManager: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), OrderManagerError>;
    async fn cleanup(&self) -> Result<(), OrderManagerError>;
    async fn new_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError>;
    async fn cancel_order(&self, order_id: u64) -> Result<(), OrderManagerError>;
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError>;
}

#[async_trait]
pub trait Executor: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), ExecutorError>;
    async fn cleanup(&self) -> Result<(), ExecutorError>;
    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError>;
    async fn cancel_order(&self, order_id: u64) -> Result<(), ExecutorError>;
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError>;
}
