use std::sync::Arc;

use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{repos::TransferRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]
pub struct TransferStore {
    transfer_repo: TransferRepo,
}

impl TransferStore {
    pub async fn insert(&self, transfer: Arc<Transfer>) -> Result<(), PersistenceError> {
        self.transfer_repo.insert(transfer.into()).await
    }

    pub async fn insert_batch(&self, transfers: Vec<Arc<Transfer>>) -> Result<(), PersistenceError> {
        let transfers_dto = transfers.into_iter().map(|t| t.into()).collect();
        self.transfer_repo.insert_batch(transfers_dto).await
    }
}
