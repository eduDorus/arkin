use async_trait::async_trait;

use arkin_core::prelude::*;
use mockall::automock;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{ExecutorError, OrderManagerError};

#[automock]
#[async_trait]
pub trait OrderManager: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), OrderManagerError>;
    async fn cleanup(&self) -> Result<(), OrderManagerError>;

    async fn place_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError>;
    async fn cancel_order(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError>;
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError>;

    async fn order_update(&self, fill: Fill) -> Result<(), OrderManagerError>;
    async fn order_status_update(&self, id: VenueOrderId, status: VenueOrderStatus) -> Result<(), OrderManagerError>;
    async fn position_update(&self, position: Position) -> Result<(), OrderManagerError>;
    async fn balance_update(&self, holding: Holding) -> Result<(), OrderManagerError>;
}

#[automock]
#[async_trait]
pub trait Executor: std::fmt::Debug + Send + Sync {
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), ExecutorError>;
    async fn cleanup(&self) -> Result<(), ExecutorError>;

    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError>;
    async fn place_orders(&self, orders: Vec<VenueOrder>) -> Result<(), ExecutorError>;

    async fn modify_order(&self, order: VenueOrder) -> Result<(), ExecutorError>;
    async fn modify_orders(&self, order: Vec<VenueOrder>) -> Result<(), ExecutorError>;

    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError>;
    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError>;
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError>;
}
