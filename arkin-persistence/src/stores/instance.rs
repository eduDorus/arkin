use std::sync::Arc;

use derive_builder::Builder;
use moka2::future::Cache;
use uuid::Uuid;

use arkin_core::Instance;

use crate::{repos::InstanceRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct InstanceStore {
    instance_repo: InstanceRepo,
    #[builder(default = "Cache::new(1000)")]
    instance_cache_id: Cache<Uuid, Arc<Instance>>,
    instance_cache_name: Cache<String, Arc<Instance>>,
}

impl InstanceStore {
    async fn update_cache(&self, instance: Arc<Instance>) {
        self.instance_cache_id.insert(instance.id, instance.clone()).await;
        self.instance_cache_name.insert(instance.name.clone(), instance).await;
    }

    async fn read_cache_by_id(&self, id: &Uuid) -> Option<Arc<Instance>> {
        self.instance_cache_id.get(id).await
    }

    async fn read_cache_by_name(&self, name: &str) -> Option<Arc<Instance>> {
        self.instance_cache_name.get(name).await
    }

    pub async fn insert(&self, instance: Arc<Instance>) -> Result<(), PersistenceError> {
        self.update_cache(instance.clone()).await;
        self.instance_repo.insert(instance.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Instance>, PersistenceError> {
        match self.read_cache_by_id(id).await {
            Some(instance) => return Ok(instance),
            None => {
                let instance_dto = self.instance_repo.read_by_id(id).await?;
                let instance: Arc<Instance> = instance_dto.into();
                self.update_cache(instance.clone()).await;
                Ok(instance)
            }
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Instance>, PersistenceError> {
        match self.read_cache_by_name(name).await {
            Some(instance) => return Ok(instance),
            None => {
                let instance_dto = self.instance_repo.read_by_name(name).await?;
                let instance: Arc<Instance> = instance_dto.into();
                self.update_cache(instance.clone()).await;
                Ok(instance)
            }
        }
    }
}
