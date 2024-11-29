use anyhow::Result;
use sqlx::PgPool;

use arkin_core::prelude::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct InstancesRepo {
    pool: PgPool,
}

impl InstancesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, instance: Instance) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO instances
            (
                id, 
                "name", 
                start_time, 
                end_time, 
                instance_type, 
                status
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            instance.id,
            instance.name,
            instance.start_time,
            instance.end_time,
            instance.instance_type as InstanceType,
            instance.status as InstanceStatus,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: Uuid) -> Result<Option<Instance>> {
        let instance = sqlx::query_as!(
            Instance,
            r#"
            SELECT 
                id,
                name,
                start_time,
                end_time,
                instance_type AS "instance_type:InstanceType",
                status AS "status:InstanceStatus"
            FROM instances 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool) // -> Vec<Country>
        .await?;
        Ok(instance)
    }

    pub async fn read_by_name(&self, name: String) -> Result<Option<Instance>> {
        let instance = sqlx::query_as!(
            Instance,
            r#"
            SELECT 
                id,
                name,
                start_time,
                end_time,
                instance_type AS "instance_type:InstanceType",
                status AS "status:InstanceStatus"
            FROM instances 
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool) // -> Vec<Country>
        .await?;
        Ok(instance)
    }

    pub async fn update(&self, instance: Instance) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE instances
            SET
                "name" = $2,
                start_time = $3,
                end_time = $4,
                instance_type = $5,
                status = $6
            WHERE id = $1;
            "#,
            instance.id,
            instance.name,
            instance.start_time,
            instance.end_time,
            instance.instance_type as InstanceType,
            instance.status as InstanceStatus,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM instances
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
    use time::OffsetDateTime;

    #[test(tokio::test)]
    async fn test_instance_repo() {
        let pool = connect_database();
        let repo = InstancesRepo::new(pool);

        let mut instance = InstanceBuilder::default()
            .id(Uuid::new_v4())
            .name("test_instance")
            .start_time(OffsetDateTime::now_utc())
            .instance_type(InstanceType::Live)
            .status(InstanceStatus::Running)
            .build()
            .unwrap();

        let result = repo.insert(instance.clone()).await;
        assert!(result.is_ok());

        let res = repo.read_by_name("test_instance".to_string()).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance.name, "test_instance");

        let res = repo.read_by_id(instance.id).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance, instance);

        instance.status = InstanceStatus::Stopped;
        let result = repo.update(instance.clone()).await;
        assert!(result.is_ok());

        let res = repo.read_by_id(instance.id).await.unwrap();
        assert!(res.is_some());
        let retrieved_instance = res.unwrap();
        assert_eq!(retrieved_instance.status, InstanceStatus::Stopped);

        let result = repo.delete(instance.id).await;
        assert!(result.is_ok());
    }
}
