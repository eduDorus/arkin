use std::sync::Arc;

use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::Instance;

use crate::{repos::InstanceRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct InstanceStore {
    instance_repo: InstanceRepo,
}

impl InstanceStore {
    pub async fn insert(&self, instance: Arc<Instance>) -> Result<(), PersistenceError> {
        info!("Inserting instance: {}", instance);
        self.instance_repo.insert(instance.into()).await?;
        Ok(())
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Instance>, PersistenceError> {
        let instance_dto = self.instance_repo.read_by_name(name).await?;
        let instance: Arc<Instance> = instance_dto.into();
        Ok(instance)
    }
}
