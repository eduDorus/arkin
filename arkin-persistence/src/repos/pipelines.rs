use std::sync::Arc;

use arkin_core::prelude::*;
use derive_builder::Builder;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(FromRow)]
pub struct PipelineDTO {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Arc<Pipeline>> for PipelineDTO {
    fn from(pipeline: Arc<Pipeline>) -> Self {
        Self {
            id: pipeline.id,
            name: pipeline.name.clone(),
            description: pipeline.description.clone(),
            created_at: pipeline.created_at,
            updated_at: pipeline.updated_at,
        }
    }
}

impl From<PipelineDTO> for Arc<Pipeline> {
    fn from(asset: PipelineDTO) -> Self {
        let asset = Pipeline {
            id: asset.id,
            name: asset.name,
            description: asset.description,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        };
        Arc::new(asset)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct PipelineRepo {
    pool: PgPool,
}

impl PipelineRepo {
    pub async fn insert(&self, asset: PipelineDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO pipelines 
            (
                id, 
                name, 
                description,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            asset.id,
            asset.name,
            asset.description,
            asset.created_at,
            asset.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<PipelineDTO, PersistenceError> {
        let pipeline = sqlx::query_as!(
            PipelineDTO,
            r#"
            SELECT
                id, 
                name, 
                description,
                created_at,
                updated_at
            FROM pipelines
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match pipeline {
            Some(pipeline) => Ok(pipeline),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<PipelineDTO, PersistenceError> {
        let pipeline = sqlx::query_as!(
            PipelineDTO,
            r#"
            SELECT
                id, 
                name, 
                description,
                created_at,
                updated_at
            FROM pipelines
            WHERE name = $1
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await?;

        match pipeline {
            Some(pipeline) => Ok(pipeline),
            None => Err(PersistenceError::NotFound),
        }
    }
}
