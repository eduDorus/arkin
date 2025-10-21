use std::sync::Arc;

use tracing::info;

use arkin_core::Instance;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::instance_repo};

pub async fn insert(ctx: &PersistenceContext, instance: Arc<Instance>) -> Result<(), PersistenceError> {
    info!("Inserting instance: {}", instance);
    instance_repo::insert(ctx, instance.into()).await
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Instance>, PersistenceError> {
    let instance_dto = instance_repo::read_by_id(ctx, id).await?;
    let instance: Arc<Instance> = instance_dto.into();
    Ok(instance)
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<Arc<Instance>, PersistenceError> {
    let instance_dto = instance_repo::read_by_name(ctx, name).await?;
    let instance: Arc<Instance> = instance_dto.into();
    Ok(instance)
}

pub async fn delete_by_id(ctx: &PersistenceContext, instance_id: Uuid) -> Result<(), PersistenceError> {
    instance_repo::delete_by_id(ctx, &instance_id).await?;
    info!("Deleted instance: {}", instance_id);
    Ok(())
}

pub async fn delete_by_name(ctx: &PersistenceContext, name: &str) -> Result<(), PersistenceError> {
    instance_repo::delete_by_name(ctx, name).await?;
    info!("Deleted instance: {}", name);
    Ok(())
}
