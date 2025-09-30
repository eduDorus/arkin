use std::sync::Arc;

use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

#[derive(Debug, Clone)]
pub struct StrategyDTO {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<Strategy> for StrategyDTO {
    fn from(strategy: Strategy) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name,
            description: strategy.description,
            created: strategy.created.into(),
            updated: strategy.updated.into(),
        }
    }
}

impl From<Arc<Strategy>> for StrategyDTO {
    fn from(strategy: Arc<Strategy>) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name.to_owned(),
            description: strategy.description.to_owned(),
            created: strategy.created.into(),
            updated: strategy.updated.into(),
        }
    }
}

impl From<StrategyDTO> for Strategy {
    fn from(strategy: StrategyDTO) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name,
            description: strategy.description,
            created: strategy.created.into(),
            updated: strategy.updated.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, strategy: StrategyDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO strategies
            (
                id, 
                name, 
                description,
                created,
                updated
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        strategy.id,
        strategy.name,
        strategy.description,
        strategy.created,
        strategy.updated
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<StrategyDTO, PersistenceError> {
    let strategy = sqlx::query_as!(
        StrategyDTO,
        r#"
            SELECT 
                id,
                name,
                description,
                created,
                updated
            FROM strategies 
            WHERE id = $1
            "#,
        id
    )
    .fetch_optional(&ctx.pg_pool) // -> Vec<Country>
    .await?;
    match strategy {
        Some(strategy) => Ok(strategy),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<StrategyDTO, PersistenceError> {
    let strategy = sqlx::query_as!(
        StrategyDTO,
        r#"
            SELECT 
                id,
                name,
                description,
                created,
                updated
            FROM strategies 
            WHERE name = $1
            "#,
        name
    )
    .fetch_optional(&ctx.pg_pool) // -> Vec<Country>
    .await?;
    match strategy {
        Some(strategy) => Ok(strategy),
        None => {
            warn!("Strategy not found: {}", name);
            Err(PersistenceError::NotFound)
        }
    }
}

pub async fn update(ctx: &PersistenceContext, strategy: StrategyDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            UPDATE strategies
            SET
                name = $2,
                description = $3,
                updated = $4
            WHERE id = $1
            "#,
        strategy.id,
        strategy.name,
        strategy.description,
        strategy.updated
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn delete(ctx: &PersistenceContext, id: &Uuid) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            DELETE FROM strategies
            WHERE id = $1
            "#,
        id
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}
