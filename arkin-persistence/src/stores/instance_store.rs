use std::sync::Arc;

use arkin_core::{Instance, InstanceListQuery, InstanceQuery};
use tracing::info;
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

/// Load all instances from database
pub async fn load_instances(ctx: &PersistenceContext) -> Result<Vec<Arc<Instance>>, PersistenceError> {
    let instance_dtos = instance_repo::list_all(ctx).await?;
    let mut instances = Vec::with_capacity(instance_dtos.len());

    for dto in instance_dtos {
        let instance: Arc<Instance> = dto.into();
        instances.push(instance);
    }

    Ok(instances)
}

/// Query instances with in-memory filtering
pub async fn query(ctx: &PersistenceContext, query: &InstanceQuery) -> Result<Arc<Instance>, PersistenceError> {
    // For now, load all and filter, since instances are few
    let all_instances = load_instances(ctx).await?;
    for instance in all_instances {
        if query.matches(&instance) {
            return Ok(instance);
        }
    }
    Err(PersistenceError::NotFound)
}

/// Query instances with in-memory filtering
pub async fn query_list(ctx: &PersistenceContext, query: &InstanceListQuery) -> Result<Vec<Arc<Instance>>, PersistenceError> {
    let all_instances = load_instances(ctx).await?;

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_instances);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Instance>> = all_instances.into_iter().filter(|instance| query.matches(instance)).collect();

    Ok(filtered)
}
