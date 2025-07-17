use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::prelude::*;
use uuid::Uuid;

use crate::{context::PersistenceContext, PersistenceError};

#[derive(FromRow)]
pub struct PipelineDTO {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl From<Arc<Pipeline>> for PipelineDTO {
    fn from(pipeline: Arc<Pipeline>) -> Self {
        Self {
            id: pipeline.id,
            name: pipeline.name.clone(),
            description: pipeline.description.clone().into(),
        }
    }
}

impl From<PipelineDTO> for Arc<Pipeline> {
    fn from(pipeline: PipelineDTO) -> Self {
        let asset = Pipeline {
            id: pipeline.id,
            name: pipeline.name,
            description: pipeline.description.unwrap_or_default(),
        };
        Arc::new(asset)
    }
}

pub async fn insert(ctx: &PersistenceContext, asset: PipelineDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO pipelines 
            (
                id, 
                name, 
                description
            ) VALUES ($1, $2, $3)
            "#,
        asset.id,
        asset.name,
        asset.description
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<PipelineDTO, PersistenceError> {
    let pipeline = sqlx::query_as!(
        PipelineDTO,
        r#"
            SELECT
                id, 
                name, 
                description
            FROM pipelines
            WHERE id = $1
            "#,
        id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match pipeline {
        Some(pipeline) => Ok(pipeline),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<PipelineDTO, PersistenceError> {
    let pipeline = sqlx::query_as!(
        PipelineDTO,
        r#"
            SELECT
                id, 
                name, 
                description
            FROM pipelines
            WHERE name = $1
            "#,
        name,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match pipeline {
        Some(pipeline) => Ok(pipeline),
        None => Err(PersistenceError::NotFound),
    }
}
