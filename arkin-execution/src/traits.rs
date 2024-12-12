use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use tokio_util::sync::CancellationToken;

use arkin_core::prelude::*;

use crate::{ExecutorError, OrderManagerError, StrategyError};

#[automock]
#[async_trait]
pub trait OrderManager: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), OrderManagerError>;

    async fn order_by_id(&self, id: ExecutionOrderId) -> Option<Arc<ExecutionOrder>>;

    async fn list_new_orders(&self) -> Vec<Arc<ExecutionOrder>>;
    async fn list_open_orders(&self) -> Vec<Arc<ExecutionOrder>>;
    async fn list_cancelling_orders(&self) -> Vec<Arc<ExecutionOrder>>;
    async fn list_cancelled_orders(&self) -> Vec<Arc<ExecutionOrder>>;
    async fn list_closed_orders(&self) -> Vec<Arc<ExecutionOrder>>;

    async fn place_order(&self, order: Arc<ExecutionOrder>) -> Result<(), OrderManagerError>;
    async fn place_orders(&self, orders: Vec<Arc<ExecutionOrder>>) -> Result<(), OrderManagerError>;

    async fn replace_order_by_id(
        &self,
        id: ExecutionOrderId,
        order: Arc<ExecutionOrder>,
    ) -> Result<(), OrderManagerError>;
    async fn replace_orders_by_instrument(
        &self,
        instrument: &Arc<Instrument>,
        order: Arc<ExecutionOrder>,
    ) -> Result<(), OrderManagerError>;

    async fn cancel_order_by_id(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError>;
    async fn cancel_orders_by_instrument(&self, instrument: &Arc<Instrument>) -> Result<(), OrderManagerError>;
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError>;

    async fn order_update(&self, fill: Arc<VenueOrderFill>) -> Result<(), OrderManagerError>;
    async fn order_status_update(
        &self,
        id: ExecutionOrderId,
        status: ExecutionOrderStatus,
    ) -> Result<(), OrderManagerError>;
}

#[automock]
#[async_trait]
pub trait ExecutionStrategy: Send + Sync {
    async fn start(&self) -> Result<(), StrategyError>;
}

#[automock]
#[async_trait]
pub trait Executor: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError>;

    // async fn subscribe(&self) -> Result<Receiver<Fill>, ExecutorError>;

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError>;
    async fn place_orders(&self, orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError>;

    async fn modify_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError>;
    async fn modify_orders(&self, order: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError>;

    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError>;
    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError>;
    async fn cancel_orders_by_instrument(&self, instrument: Arc<Instrument>) -> Result<(), ExecutorError>;
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError>;
}
