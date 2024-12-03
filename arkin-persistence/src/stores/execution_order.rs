use std::sync::Arc;

use arkin_core::ExecutionOrder;
use derive_builder::Builder;
use uuid::Uuid;

use crate::{repos::ExecutionOrderRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
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
