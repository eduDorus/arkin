use std::sync::Arc;

use arkin_core::{Instance, InstanceType};
use sqlx::PgPool;
use typed_builder::TypedBuilder;

use uuid::Uuid;

use crate::PersistenceError;

#[derive(Debug, Clone)]
pub struct InstanceDTO {
    pub id: Uuid,
    pub name: String,
    pub instance_type: InstanceType,
}

impl From<Instance> for InstanceDTO {
    fn from(instance: Instance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            instance_type: instance.instance_type,
        }
    }
}

impl From<Arc<Instance>> for InstanceDTO {
    fn from(instance: Arc<Instance>) -> Self {
        Self {
            id: instance.id,
            name: instance.name.clone(),
            instance_type: instance.instance_type,
        }
    }
}

impl From<InstanceDTO> for Arc<Instance> {
    fn from(instance: InstanceDTO) -> Self {
        let new_instance = Instance::builder()
            .id(instance.id)
            .name(instance.name)
            .instance_type(instance.instance_type)
            .build();
        Arc::new(new_instance)
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct InstanceRepo {
    pool: PgPool,
}

impl InstanceRepo {
    pub async fn insert(&self, instance: InstanceDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO instances
            (
                id, 
                "name", 
                instance_type
            ) VALUES ($1, $2, $3)
            "#,
            instance.id,
            instance.name,
            instance.instance_type as InstanceType,
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
                instance_type AS "instance_type:InstanceType"
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
                instance_type AS "instance_type:InstanceType"
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
