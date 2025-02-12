use std::sync::Arc;

use async_trait::async_trait;

use arkin_core::prelude::*;

use crate::errors::ExecutorError;

#[async_trait]
pub trait ExecutorService: RunnableService + Executor {}

#[async_trait]
pub trait Executor: std::fmt::Debug + Send + Sync {
    async fn get_account(&self) -> Result<(), ExecutorError>;
    async fn get_balances(&self) -> Result<(), ExecutorError>;
    async fn get_positions(&self) -> Result<(), ExecutorError>;

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError>;
    async fn place_orders(&self, orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError>;

    async fn modify_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError>;
    async fn modify_orders(&self, order: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError>;

    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError>;
    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError>;
    async fn cancel_orders_by_instrument(&self, instrument: Arc<Instrument>) -> Result<(), ExecutorError>;
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError>;
}
