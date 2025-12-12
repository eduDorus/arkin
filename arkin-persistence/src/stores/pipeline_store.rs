use std::sync::Arc;

use arkin_core::{Pipeline, PipelineListQuery, PipelineQuery};
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::pipeline_repo};

pub async fn insert(ctx: &PersistenceContext, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
    pipeline_repo::insert(ctx, pipeline.into()).await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
    let pipeline_dto = pipeline_repo::read_by_id(ctx, id).await?;
    let pipeline: Arc<Pipeline> = pipeline_dto.into();
    Ok(pipeline)
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
    let pipeline_dto = pipeline_repo::read_by_name(ctx, name).await?;
    let pipeline: Arc<Pipeline> = pipeline_dto.into();
    Ok(pipeline)
}

/// Load all pipelines from database
pub async fn load_pipelines(ctx: &PersistenceContext) -> Result<Vec<Arc<Pipeline>>, PersistenceError> {
    let pipeline_dtos = pipeline_repo::list_all(ctx).await?;
    let mut pipelines = Vec::with_capacity(pipeline_dtos.len());

    for dto in pipeline_dtos {
        let pipeline: Arc<Pipeline> = dto.into();
        pipelines.push(pipeline);
    }

    Ok(pipelines)
}

/// Query pipelines with in-memory filtering
pub async fn query(ctx: &PersistenceContext, query: &PipelineQuery) -> Result<Arc<Pipeline>, PersistenceError> {
    // For now, load all and filter, since pipelines are few
    let all_pipelines = load_pipelines(ctx).await?;
    for pipeline in all_pipelines {
        if query.matches(&pipeline) {
            return Ok(pipeline);
        }
    }
    Err(PersistenceError::NotFound)
}

/// Query pipelines with in-memory filtering
pub async fn query_list(ctx: &PersistenceContext, query: &PipelineListQuery) -> Result<Vec<Arc<Pipeline>>, PersistenceError> {
    let all_pipelines = load_pipelines(ctx).await?;

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_pipelines);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Pipeline>> = all_pipelines.into_iter().filter(|pipeline| query.matches(pipeline)).collect();

    Ok(filtered)
}
