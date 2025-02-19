use std::sync::Arc;

use arkin_core::{Instance, InstanceStatus, InstanceType};
use sqlx::PgPool;
use typed_builder::TypedBuilder;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct InstanceDTO {
    pub id: Uuid,
    pub name: String,
    pub start_time: OffsetDateTime,
    pub end_time: Option<OffsetDateTime>,
    pub instance_type: InstanceType,
    pub status: InstanceStatus,
}

impl From<Instance> for InstanceDTO {
    fn from(instance: Instance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            start_time: instance.start_time,
            end_time: instance.end_time,
            instance_type: instance.instance_type,
            status: instance.status,
        }
    }
}

impl From<Arc<Instance>> for InstanceDTO {
    fn from(instance: Arc<Instance>) -> Self {
        Self {
            id: instance.id,
            name: instance.name.clone(),
            start_time: instance.start_time,
            end_time: instance.end_time,
            instance_type: instance.instance_type,
            status: instance.status,
        }
    }
}

impl From<InstanceDTO> for Arc<Instance> {
    fn from(instance: InstanceDTO) -> Self {
        let instance = Instance {
            id: instance.id,
            name: instance.name,
            start_time: instance.start_time,
            end_time: instance.end_time,
            instance_type: instance.instance_type,
            status: instance.status,
        };
        Arc::new(instance)
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct InstanceRepo {
    pool: PgPool,
}

impl InstanceRepo {
    pub async fn create_table(&self) -> Result<(), PersistenceError> {
        // Create the instance_type enum if it does not exist
        sqlx::query!(
            r#"
            DO $$
            BEGIN
              IF NOT EXISTS (
                SELECT 1
                FROM pg_type
                WHERE typname = 'instance_type'
              ) THEN
                CREATE TYPE instance_type AS ENUM ('live', 'simulation');
              END IF;
            END
            $$;
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create the instance_status enum if it does not exist
        sqlx::query!(
            r#"
            DO $$
            BEGIN
              IF NOT EXISTS (
                SELECT 1
                FROM pg_type
                WHERE typname = 'instance_status'
              ) THEN
                CREATE TYPE instance_status AS ENUM ('new', 'running', 'stopped', 'completed', 'failed');
              END IF;
            END
            $$;
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert(&self, instance: InstanceDTO) -> Result<(), PersistenceError> {
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

    pub async fn read_by_id(&self, id: &Uuid) -> Result<InstanceDTO, PersistenceError> {
        let instance = sqlx::query_as!(
            InstanceDTO,
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
        match instance {
            Some(instance) => Ok(instance),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<InstanceDTO, PersistenceError> {
        let instance = sqlx::query_as!(
            InstanceDTO,
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
        match instance {
            Some(instance) => Ok(instance),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn update(&self, instance: InstanceDTO) -> Result<(), PersistenceError> {
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

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
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
        let repo = InstanceRepo::builder().pool(pool).build();

        let mut instance = Instance::builder()
            .id(Uuid::new_v4())
            .name("test_instance".into())
            .start_time(OffsetDateTime::now_utc())
            .instance_type(InstanceType::Live)
            .status(InstanceStatus::Running)
            .build();

        let wrapped_instance = Arc::new(instance.clone());

        let result = repo.insert(wrapped_instance.clone().into()).await;
        assert!(result.is_ok());

        let res = repo.read_by_name("test_instance").await.unwrap();
        assert_eq!(res.name, "test_instance");

        let res = repo.read_by_id(&instance.id).await.unwrap();
        assert_eq!(Into::<Arc<Instance>>::into(res), wrapped_instance);

        instance.status = InstanceStatus::Stopped;
        let result = repo.update(Arc::new(instance.clone()).into()).await;
        assert!(result.is_ok());

        let res = repo.read_by_id(&instance.id).await.unwrap();
        assert_eq!(res.status, InstanceStatus::Stopped);

        let result = repo.delete(&instance.id).await;
        assert!(result.is_ok());
    }
}
