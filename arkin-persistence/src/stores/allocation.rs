use std::sync::Arc;

use arkin_core::Allocation;
use typed_builder::TypedBuilder;

use crate::{repos::AllocationRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct AllocationStore {
    allocation_repo: AllocationRepo,
}

impl AllocationStore {
    pub async fn insert(&self, allocation: Arc<Allocation>) -> Result<(), PersistenceError> {
        self.allocation_repo.insert(allocation.into()).await
    }

    pub async fn insert_batch(&self, allocation: Vec<Arc<Allocation>>) -> Result<(), PersistenceError> {
        let allocation_dto: Vec<_> = allocation.into_iter().map(|signal| signal.into()).collect();
        self.allocation_repo.insert_batch(allocation_dto).await
    }
}
