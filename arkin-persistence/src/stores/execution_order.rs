use std::sync::Arc;

use arkin_core::ExecutionOrder;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{repos::ExecutionOrderRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct ExecutionOrderStore {
    execution_order_repo: ExecutionOrderRepo,
}

impl ExecutionOrderStore {
    pub async fn insert(&self, order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
        self.execution_order_repo.insert(order.into()).await
    }

    pub async fn update(&self, order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
        self.execution_order_repo.update(order.into()).await
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
        self.execution_order_repo.delete(id).await
    }
}
