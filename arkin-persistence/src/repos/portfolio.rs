use std::sync::Arc;

use arkin_core::prelude::*;
use typed_builder::TypedBuilder;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(FromRow)]
pub struct PortfolioDTO {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Arc<Portfolio>> for PortfolioDTO {
    fn from(pipeline: Arc<Portfolio>) -> Self {
        Self {
            id: pipeline.id,
            name: pipeline.name.clone(),
            description: pipeline.description.clone(),
            created_at: pipeline.created_at,
            updated_at: pipeline.updated_at,
        }
    }
}

impl From<PortfolioDTO> for Arc<Portfolio> {
    fn from(asset: PortfolioDTO) -> Self {
        let asset = Portfolio {
            id: asset.id,
            name: asset.name,
            description: asset.description,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        };
        Arc::new(asset)
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct PortfolioRepo {
    pool: PgPool,
}

impl PortfolioRepo {
    pub async fn insert(&self, portfolio: PortfolioDTO) -> Result<(), PersistenceError> {
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
            portfolio.id,
            portfolio.name,
            portfolio.description,
            portfolio.created_at,
            portfolio.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<PortfolioDTO, PersistenceError> {
        let portfolio = sqlx::query_as!(
            PortfolioDTO,
            r#"
            SELECT
                id, 
                name, 
                description,
                created_at,
                updated_at
            FROM portfolios
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match portfolio {
            Some(pipeline) => Ok(pipeline),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<PortfolioDTO, PersistenceError> {
        let portfolio = sqlx::query_as!(
            PortfolioDTO,
            r#"
            SELECT
                id, 
                name, 
                description,
                created_at,
                updated_at
            FROM portfolios
            WHERE name = $1
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await?;

        match portfolio {
            Some(pipeline) => Ok(pipeline),
            None => Err(PersistenceError::NotFound),
        }
    }
}
