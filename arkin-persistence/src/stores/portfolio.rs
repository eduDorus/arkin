use std::sync::Arc;

use derive_builder::Builder;
use moka2::future::Cache;
use uuid::Uuid;

use arkin_core::Portfolio;

use crate::{repos::PortfolioRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct PortfolioStore {
    portfolio_repo: PortfolioRepo,
    #[builder(default = "Cache::new(1000)")]
    portfolio_cache_id: Cache<Uuid, Arc<Portfolio>>,
    portfolio_cache_name: Cache<String, Arc<Portfolio>>,
}

impl PortfolioStore {
    async fn update_cache(&self, asset: Arc<Portfolio>) {
        self.portfolio_cache_id.insert(asset.id, asset.clone()).await;
        self.portfolio_cache_name.insert(asset.name.clone(), asset).await;
    }

    async fn read_cache_by_id(&self, id: &Uuid) -> Option<Arc<Portfolio>> {
        self.portfolio_cache_id.get(id).await
    }

    async fn read_cache_by_name(&self, name: &str) -> Option<Arc<Portfolio>> {
        self.portfolio_cache_name.get(name).await
    }

    pub async fn insert(&self, portfolio: Arc<Portfolio>) -> Result<(), PersistenceError> {
        self.update_cache(portfolio.clone()).await;
        self.portfolio_repo.insert(portfolio.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Portfolio>, PersistenceError> {
        match self.read_cache_by_id(id).await {
            Some(portfolio) => return Ok(portfolio),
            None => {
                let portfolio_dto = self.portfolio_repo.read_by_id(id).await?;
                let portfolio: Arc<Portfolio> = portfolio_dto.into();
                self.update_cache(portfolio.clone()).await;
                Ok(portfolio)
            }
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Portfolio>, PersistenceError> {
        match self.read_cache_by_name(name).await {
            Some(portfolio) => return Ok(portfolio),
            None => {
                let portfolio_dto = self.portfolio_repo.read_by_name(name).await?;
                let portfolio: Arc<Portfolio> = portfolio_dto.into();
                self.update_cache(portfolio.clone()).await;
                Ok(portfolio)
            }
        }
    }
}
