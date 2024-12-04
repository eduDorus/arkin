use std::sync::Arc;

use tokio::sync::Mutex;
use typed_builder::TypedBuilder;

use arkin_core::Transaction;
use tracing::error;

use crate::{repos::TransactionRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct TransactionStore {
    transaction_repo: TransactionRepo,
    #[builder(default)]
    transaction_buffer: Arc<Mutex<Vec<Arc<Transaction>>>>,
    buffer_size: usize,
}

impl TransactionStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        // Lock and extract ticks without cloning
        let transactions = {
            let mut lock = self.transaction_buffer.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        let transactions = transactions.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if let Err(e) = self.transaction_repo.insert_batch(transactions).await {
            error!("Failed to flush transactions: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), PersistenceError> {
        let should_commit = {
            let lock = self.transaction_buffer.lock().await;
            lock.len() >= self.buffer_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, transaction: Arc<Transaction>) -> Result<(), PersistenceError> {
        self.transaction_repo.insert(transaction.into()).await
    }

    pub async fn insert_buffered(&self, transaction: Arc<Transaction>) -> Result<(), PersistenceError> {
        {
            let mut lock = self.transaction_buffer.lock().await; // Wait for lock
            lock.push(transaction);
        }
        self.commit().await
    }

    pub async fn insert_buffered_vec(&self, transactions: Vec<Arc<Transaction>>) -> Result<(), PersistenceError> {
        for transaction in transactions {
            self.insert_buffered(transaction).await?;
        }
        Ok(())
    }
}
