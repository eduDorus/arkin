use std::sync::Arc;

use derive_builder::Builder;
use moka2::future::Cache;
use uuid::Uuid;

use arkin_core::Asset;

use crate::{repos::AssetRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct AssetStore {
    asset_repo: AssetRepo,
    #[builder(default = "Cache::new(1000)")]
    asset_cache_id: Cache<Uuid, Arc<Asset>>,
    asset_cache_symbol: Cache<String, Arc<Asset>>,
}

impl AssetStore {
    async fn update_cache(&self, asset: Arc<Asset>) {
        self.asset_cache_id.insert(asset.id, asset.clone()).await;
        self.asset_cache_symbol.insert(asset.name.clone(), asset).await;
    }

    async fn read_cache_by_id(&self, id: &Uuid) -> Option<Arc<Asset>> {
        self.asset_cache_id.get(id).await
    }

    async fn read_cache_by_symbol(&self, symbol: &str) -> Option<Arc<Asset>> {
        self.asset_cache_symbol.get(symbol).await
    }

    pub async fn insert(&self, asset: Arc<Asset>) -> Result<(), PersistenceError> {
        self.update_cache(asset.clone()).await;
        self.asset_repo.insert(asset.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Asset>, PersistenceError> {
        match self.read_cache_by_id(id).await {
            Some(asset) => return Ok(asset),
            None => {
                let asset_dto = self.asset_repo.read_by_id(id).await?;
                let asset: Arc<Asset> = asset_dto.into();
                self.update_cache(asset.clone()).await;
                Ok(asset)
            }
        }
    }

    pub async fn read_by_symbol(&self, symbol: &str) -> Result<Arc<Asset>, PersistenceError> {
        match self.read_cache_by_symbol(symbol).await {
            Some(asset) => return Ok(asset),
            None => {
                let asset_dto = self.asset_repo.read_by_symbol(symbol).await?;
                let asset: Arc<Asset> = asset_dto.into();
                self.update_cache(asset.clone()).await;
                Ok(asset)
            }
        }
    }
}
