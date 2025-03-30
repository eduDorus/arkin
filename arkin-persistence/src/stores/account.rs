use std::sync::Arc;

use typed_builder::TypedBuilder;

use arkin_core::AccountUpdate;

use crate::{errors::PersistenceError, repos::AccountRepo};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AccountStore {
    account_repo: AccountRepo,
}

impl AccountStore {
    pub async fn insert(&self, account: Arc<AccountUpdate>) -> Result<(), PersistenceError> {
        let account = account.account.clone();
        self.account_repo.insert(account.into()).await?;
        Ok(())
    }

    // pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Account>, PersistenceError> {
    //     let account_dto = self.account_repo.read_by_id(id).await?;
    //     let account: Arc<Account> = account_dto.into();
    //     Ok(account)
    // }

    // pub async fn read_by_name(&self, name: &str) -> Result<Arc<Account>, PersistenceError> {
    //     match self.read_cache_by_name(name).await {
    //         Some(account) => return Ok(account),
    //         None => {
    //             let account_dto = self.account_repo.read_by_name(name).await?;
    //             let account: Arc<Account> = account_dto.into();
    //             self.update_cache(account.clone()).await;
    //             Ok(account)
    //         }
    //     }
    // }
}
