use std::sync::Arc;

use moka2::future::Cache;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::Account;

use crate::{errors::PersistenceError, repos::AccountRepo};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AccountStore {
    account_repo: AccountRepo,
    #[builder(default = Cache::new(1000))]
    account_cache_id: Cache<Uuid, Arc<Account>>,
    #[builder(default = Cache::new(1000))]
    account_cache_name: Cache<String, Arc<Account>>,
}

impl AccountStore {
    async fn update_cache(&self, asset: Arc<Account>) {
        self.account_cache_id.insert(asset.id, asset.clone()).await;
        self.account_cache_name.insert(asset.name.clone(), asset).await;
    }

    async fn read_cache_by_id(&self, id: &Uuid) -> Option<Arc<Account>> {
        self.account_cache_id.get(id).await
    }

    async fn read_cache_by_name(&self, name: &str) -> Option<Arc<Account>> {
        self.account_cache_name.get(name).await
    }

    pub async fn insert(&self, account: Arc<Account>) -> Result<(), PersistenceError> {
        self.update_cache(account.clone()).await;
        self.account_repo.insert(account.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Account>, PersistenceError> {
        match self.read_cache_by_id(id).await {
            Some(account) => return Ok(account),
            None => {
                let account_dto = self.account_repo.read_by_id(id).await?;
                let account: Arc<Account> = account_dto.into();
                self.update_cache(account.clone()).await;
                Ok(account)
            }
        }
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Account>, PersistenceError> {
        match self.read_cache_by_name(name).await {
            Some(account) => return Ok(account),
            None => {
                let account_dto = self.account_repo.read_by_name(name).await?;
                let account: Arc<Account> = account_dto.into();
                self.update_cache(account.clone()).await;
                Ok(account)
            }
        }
    }
}
