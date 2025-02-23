use std::sync::Arc;

use tokio::sync::Mutex;
use typed_builder::TypedBuilder;

use tracing::error;

use crate::{repos::TransferRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct TransferStore {
    transfer_repo: TransferRepo,
    #[builder(default)]
    transfer_buffer: Arc<Mutex<Vec<Arc<Transfer>>>>,
    buffer_size: usize,
}

impl TransferStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        // Lock and extract ticks without cloning
        let transfers = {
            let mut lock = self.transfer_buffer.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        let transfers = transfers.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if let Err(e) = self.transfer_repo.insert_batch(transfers).await {
            error!("Failed to flush transfers: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), PersistenceError> {
        let should_commit = {
            let lock = self.transfer_buffer.lock().await;
            lock.len() >= self.buffer_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, transfer: Arc<Transfer>) -> Result<(), PersistenceError> {
        self.transfer_repo.insert(transfer.into()).await
    }

    pub async fn insert_buffered(&self, transfer: Arc<Transfer>) -> Result<(), PersistenceError> {
        {
            let mut lock = self.transfer_buffer.lock().await; // Wait for lock
            lock.push(transfer);
        }
        self.commit().await
    }

    pub async fn insert_buffered_vec(&self, transfers: Vec<Arc<Transfer>>) -> Result<(), PersistenceError> {
        for transfer in transfers {
            self.insert_buffered(transfer).await?;
        }
        Ok(())
    }
}
