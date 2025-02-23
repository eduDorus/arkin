use std::sync::Arc;

use sqlx::PgPool;
use tracing::warn;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct StrategyDTO {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl From<Strategy> for StrategyDTO {
    fn from(strategy: Strategy) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name,
            description: strategy.description,
        }
    }
}

impl From<Arc<Strategy>> for StrategyDTO {
    fn from(strategy: Arc<Strategy>) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name.to_owned(),
            description: strategy.description.to_owned(),
        }
    }
}

impl From<StrategyDTO> for Strategy {
    fn from(strategy: StrategyDTO) -> Self {
        Self {
            id: strategy.id,
            name: strategy.name,
            description: strategy.description,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct StrategyRepo {
    pool: PgPool,
}

impl StrategyRepo {
    pub async fn insert(&self, strategy: StrategyDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO strategies
            (
                id, 
                name, 
                description
            ) VALUES ($1, $2, $3)
            "#,
            strategy.id,
            strategy.name,
            strategy.description,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<StrategyDTO, PersistenceError> {
        let strategy = sqlx::query_as!(
            StrategyDTO,
            r#"
            SELECT 
                id,
                name,
                description
            FROM strategies 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool) // -> Vec<Country>
        .await?;
        match strategy {
            Some(strategy) => Ok(strategy),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<StrategyDTO, PersistenceError> {
        let strategy = sqlx::query_as!(
            StrategyDTO,
            r#"
            SELECT 
                id,
                name,
                description
            FROM strategies 
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool) // -> Vec<Country>
        .await?;
        match strategy {
            Some(strategy) => Ok(strategy),
            None => {
                warn!("Strategy not found: {}", name);
                Err(PersistenceError::NotFound)
            }
        }
    }

    pub async fn update(&self, strategy: StrategyDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            UPDATE strategies
            SET
                name = $2,
                description = $3
            WHERE id = $1
            "#,
            strategy.id,
            strategy.name,
            strategy.description
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            DELETE FROM strategies
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_utils::connect_database;

    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_strategy_repo() {
        let pool = connect_database();
        let repo = StrategyRepo::builder().pool(pool).build();

        let mut strategy = Strategy::builder()
            .name("test_strategy".into())
            .description(Some("test_description".to_string()))
            .build();

        let result = repo.insert(strategy.clone().into()).await;
        assert!(result.is_ok());

        let res = repo.read_by_name("test_strategy").await.unwrap();
        assert_eq!(Strategy::from(res), strategy);

        let res = repo.read_by_id(&strategy.id).await.unwrap();
        assert_eq!(Strategy::from(res), strategy);

        strategy.name = "updated_name".to_string();
        let result = repo.update(strategy.clone().into()).await;
        assert!(result.is_ok());

        let res = repo.read_by_id(&strategy.id).await.unwrap();
        assert_eq!(res.name, "updated_name");

        let result = repo.delete(&strategy.id).await;
        assert!(result.is_ok());
    }
}
