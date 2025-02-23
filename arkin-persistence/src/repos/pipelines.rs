use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::PersistenceError;

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

#[derive(Debug, Clone, TypedBuilder)]

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
                description
            ) VALUES ($1, $2, $3)
            "#,
            asset.id,
            asset.name,
            asset.description
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
                description
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
                description
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
