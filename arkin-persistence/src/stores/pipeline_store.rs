use std::sync::Arc;

use arkin_core::Pipeline;
use uuid::Uuid;

use crate::{context::PersistenceContext, repos::pg::pipeline_repo, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
    pipeline_repo::insert(ctx, pipeline.into()).await
}

pub async fn read_by_name_or_create(ctx: &PersistenceContext, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
    match pipeline_repo::read_by_name(ctx, name).await {
        Ok(pipeline_dto) => {
            let pipeline: Arc<Pipeline> = pipeline_dto.into();
            Ok(pipeline)
        }
        Err(_) => {
            let pipeline: Arc<Pipeline> = Pipeline::builder()
                .name(name.to_string())
                .description("Generated pipeline".to_string())
                .build()
                .into();
            pipeline_repo::insert(ctx, pipeline.clone().into()).await?;
            Ok(pipeline)
        }
    }
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
