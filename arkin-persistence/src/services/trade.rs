use std::sync::Arc;

use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{
    stores::{ExecutionOrderStore, SignalStore, VenueOrderStore},
    PersistenceError,
};

#[derive(Debug, Clone, TypedBuilder)]

pub struct TradeService {
    pub signal_store: SignalStore,
    pub execution_order_store: ExecutionOrderStore,
    pub venue_order_store: VenueOrderStore,
}

impl TradeService {
    pub async fn insert_execution_order(&self, execution_order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
        self.execution_order_store.insert(execution_order).await
    }

    pub async fn update_execution_order(&self, execution_order: Arc<ExecutionOrder>) -> Result<(), PersistenceError> {
        self.execution_order_store.update(execution_order).await
    }

    pub async fn delete_execution_order(&self, id: &Uuid) -> Result<(), PersistenceError> {
        self.execution_order_store.delete(id).await
    }
}
