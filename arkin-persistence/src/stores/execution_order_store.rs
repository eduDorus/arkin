use std::sync::Arc;

use arkin_core::ExecutionOrder;
use uuid::Uuid;

use crate::{context::PersistenceContext, repos::pg::execution_order_repo, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
    execution_order_repo::insert(ctx, order.into()).await
}

pub async fn update(ctx: &PersistenceContext, order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
    execution_order_repo::update(ctx, order.into()).await
}

pub async fn delete(ctx: &PersistenceContext, id: &Uuid) -> Result<(), PersistenceError> {
    execution_order_repo::delete(ctx, id).await
}
