use anyhow::Result;
use sqlx::PgPool;

use arkin_core::prelude::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct StrategiesRepo {
    pool: PgPool,
}

impl StrategiesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, strategy: Strategy) -> Result<()> {
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

    pub async fn read_by_id(&self, id: Uuid) -> Result<Option<Strategy>> {
        let strategy = sqlx::query_as!(
            Strategy,
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
        Ok(strategy)
    }

    pub async fn read_by_name(&self, name: String) -> Result<Option<Strategy>> {
        let strategy = sqlx::query_as!(
            Strategy,
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
        Ok(strategy)
    }

    pub async fn update(&self, strategy: Strategy) -> Result<()> {
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

    pub async fn delete(&self, id: Uuid) -> Result<()> {
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
        let repo = StrategiesRepo::new(pool);

        let mut strategy = StrategyBuilder::default()
            .name("test_strategy")
            .description(Some("test_description".to_string()))
            .build()
            .unwrap();

        let result = repo.insert(strategy.clone()).await;
        assert!(result.is_ok());

        let res = repo.read_by_name("test_strategy".to_string()).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance, strategy);

        let res = repo.read_by_id(strategy.id).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance, strategy);

        strategy.name = "updated_name".to_string();
        let result = repo.update(strategy.clone()).await;
        assert!(result.is_ok());

        let res = repo.read_by_id(strategy.id).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance.name, "updated_name");

        let result = repo.delete(strategy.id).await;
        assert!(result.is_ok());
    }
}
