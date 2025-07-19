use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, repos::pg::transfer_repo, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, transfer: Arc<Transfer>) -> Result<(), PersistenceError> {
    transfer_repo::insert(ctx, transfer.into()).await
}

pub async fn insert_batch(ctx: &PersistenceContext, transfers: Vec<Arc<Transfer>>) -> Result<(), PersistenceError> {
    let transfers_dto = transfers.into_iter().map(|t| t.into()).collect();
    transfer_repo::insert_batch(ctx, transfers_dto).await
}
