use std::sync::Arc;

use arkin_core::Pipeline;
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
