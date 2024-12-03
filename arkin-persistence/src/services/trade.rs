use std::sync::Arc;

use derive_builder::Builder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{
    stores::{AllocationStore, ExecutionOrderStore, PortfolioStore, SignalStore, TransactionStore, VenueOrderStore},
    PersistenceError,
};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct TradeService {
    pub portfolio_store: PortfolioStore,
    pub transaction_store: TransactionStore,
    pub signal_store: SignalStore,
    pub allocation_store: AllocationStore,
    pub execution_order_store: ExecutionOrderStore,
    pub venue_order_store: VenueOrderStore,
}

impl TradeService {
    pub async fn insert_portfolio(&self, portfolio: Arc<Portfolio>) -> Result<(), PersistenceError> {
        self.portfolio_store.insert(portfolio).await
    }

    pub async fn read_portfolio_by_id(&self, id: &Uuid) -> Result<Arc<Portfolio>, PersistenceError> {
        self.portfolio_store.read_by_id(id).await
    }

    pub async fn read_portfolio_by_name(&self, name: &str) -> Result<Arc<Portfolio>, PersistenceError> {
        self.portfolio_store.read_by_name(name).await
    }

    pub async fn insert_transaction(&self, transaction: Arc<Transaction>) -> Result<(), PersistenceError> {
        self.transaction_store.insert(transaction).await
    }

    pub async fn insert_transaction_buffered(&self, transaction: Arc<Transaction>) -> Result<(), PersistenceError> {
        self.transaction_store.insert_buffered(transaction).await
    }

    pub async fn insert_transaction_buffered_vec(
        &self,
        transactions: Vec<Arc<Transaction>>,
    ) -> Result<(), PersistenceError> {
        self.transaction_store.insert_buffered_vec(transactions).await
    }

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
