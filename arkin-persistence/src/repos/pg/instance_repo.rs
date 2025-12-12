use std::sync::Arc;

use arkin_core::{Instance, InstanceType};

use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

#[derive(Debug, Clone)]
pub struct InstanceDTO {
    pub id: Uuid,
    pub name: String,
    pub instance_type: InstanceType,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<Instance> for InstanceDTO {
    fn from(instance: Instance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            instance_type: instance.instance_type,
            created: instance.created.into(),
            updated: instance.updated.into(),
        }
    }
}

impl From<Arc<Instance>> for InstanceDTO {
    fn from(instance: Arc<Instance>) -> Self {
        Self {
            id: instance.id,
            name: instance.name.clone(),
            instance_type: instance.instance_type,
            created: instance.created.into(),
            updated: instance.updated.into(),
        }
    }
}

impl From<InstanceDTO> for Arc<Instance> {
    fn from(instance: InstanceDTO) -> Self {
        let new_instance = Instance::builder()
            .id(instance.id)
            .name(instance.name)
            .instance_type(instance.instance_type)
            .created(instance.created.into())
            .updated(instance.updated.into())
            .build();
        Arc::new(new_instance)
    }
}

pub async fn insert(ctx: &PersistenceContext, instance: InstanceDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO instances
            (
                id, 
                "name", 
                instance_type,
                created,
                updated
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        instance.id,
        instance.name,
        instance.instance_type as InstanceType,
        instance.created,
        instance.updated,
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<InstanceDTO, PersistenceError> {
    let instance = sqlx::query_as!(
        InstanceDTO,
        r#"
            SELECT 
                id,
                name,
                instance_type AS "instance_type:InstanceType",
                created,
                updated
            FROM instances 
            WHERE id = $1
            "#,
        id
    )
    .fetch_optional(&ctx.pg_pool) // -> Vec<Country>
    .await?;
    match instance {
        Some(instance) => Ok(instance),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<InstanceDTO, PersistenceError> {
    let instance = sqlx::query_as!(
        InstanceDTO,
        r#"
            SELECT 
                id,
                name,
                instance_type AS "instance_type:InstanceType",
                created,
                updated
            FROM instances 
            WHERE name = $1
            "#,
        name
    )
    .fetch_optional(&ctx.pg_pool) // -> Vec<Country>
    .await?;
    match instance {
        Some(instance) => Ok(instance),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn delete_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            DELETE FROM instances
            WHERE id = $1
            "#,
        id
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn delete_by_name(ctx: &PersistenceContext, name: &str) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            DELETE FROM instances
            WHERE name = $1
            "#,
        name
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn list_all(ctx: &PersistenceContext) -> Result<Vec<InstanceDTO>, PersistenceError> {
    let instances = sqlx::query_as!(
        InstanceDTO,
        r#"
            SELECT
                id,
                name,
                instance_type AS "instance_type:InstanceType",
                created,
                updated
            FROM instances
            ORDER BY name
            "#,
    )
    .fetch_all(&ctx.pg_pool)
    .await?;

    Ok(instances)
}
