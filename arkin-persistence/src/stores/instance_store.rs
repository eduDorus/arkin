use std::sync::Arc;

use tracing::info;

use arkin_core::Instance;
use uuid::Uuid;

use crate::{context::PersistenceContext, repos::pg::instance_repo, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, instance: Arc<Instance>) -> Result<(), PersistenceError> {
    info!("Inserting instance: {}", instance);
    instance_repo::insert(ctx, instance.into()).await?;
    Ok(())
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<Arc<Instance>, PersistenceError> {
    let instance_dto = instance_repo::read_by_name(ctx, name).await?;
    let instance: Arc<Instance> = instance_dto.into();
    Ok(instance)
}

pub async fn delete(ctx: &PersistenceContext, instance_id: Uuid) -> Result<(), PersistenceError> {
    instance_repo::delete(ctx, &instance_id).await?;
    info!("Deleted instance: {}", instance_id);
    Ok(())
}
