use std::sync::Arc;

use arkin_core::ExecutionOrder;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::{
    context::PersistenceContext,
    repos::ch::execution_order_repo::{self, ExecutionOrderClickhouseDTO},
};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    execution_order_repo::create_table(ctx).await
}

pub async fn insert(ctx: &PersistenceContext, order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
    execution_order_repo::insert(ctx, ExecutionOrderClickhouseDTO::from_model(&order, ctx.instance.id)).await
}

pub async fn insert_batch(ctx: &PersistenceContext, orders: &[Arc<ExecutionOrder>]) -> Result<(), PersistenceError> {
    let dtos: Vec<_> = orders
        .iter()
        .map(|o| ExecutionOrderClickhouseDTO::from_model(o, ctx.instance.id))
        .collect();
    execution_order_repo::insert_batch(ctx, &dtos).await
}
