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

    async fn order_by_id(&self, id: ExecutionOrderId) -> Option<ExecutionOrder>;

    async fn list_new_orders(&self) -> Vec<ExecutionOrder>;
    async fn list_open_orders(&self) -> Vec<ExecutionOrder>;
    async fn list_cancelling_orders(&self) -> Vec<ExecutionOrder>;
    async fn list_cancelled_orders(&self) -> Vec<ExecutionOrder>;
    async fn list_closed_orders(&self) -> Vec<ExecutionOrder>;

    async fn place_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError>;
    async fn place_orders(&self, orders: Vec<ExecutionOrder>) -> Result<(), OrderManagerError>;

    async fn replace_order_by_id(&self, id: ExecutionOrderId, order: ExecutionOrder) -> Result<(), OrderManagerError>;
    async fn replace_orders_by_instrument(
        &self,
        instrument: &Arc<Instrument>,
        order: ExecutionOrder,
    ) -> Result<(), OrderManagerError>;

    async fn cancel_order_by_id(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError>;
    async fn cancel_orders_by_instrument(&self, instrument: &Arc<Instrument>) -> Result<(), OrderManagerError>;
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError>;

    async fn order_update(&self, fill: Fill) -> Result<(), OrderManagerError>;
    async fn order_status_update(
        &self,
        id: ExecutionOrderId,
        status: ExecutionOrderStatus,
    ) -> Result<(), OrderManagerError>;
    async fn position_update(&self, position: Position) -> Result<(), OrderManagerError>;
    async fn balance_update(&self, holding: Holding) -> Result<(), OrderManagerError>;
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

    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError>;
    async fn place_orders(&self, orders: Vec<VenueOrder>) -> Result<(), ExecutorError>;

    async fn modify_order(&self, order: VenueOrder) -> Result<(), ExecutorError>;
    async fn modify_orders(&self, order: Vec<VenueOrder>) -> Result<(), ExecutorError>;

    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError>;
    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError>;
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError>;
}
