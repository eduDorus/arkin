use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, repos::pg::account_repo};

pub async fn insert(ctx: &PersistenceContext, account: Arc<Account>) -> Result<(), PersistenceError> {
    account_repo::insert(ctx, account.into()).await
}

// pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Account>, PersistenceError> {
//     let account_dto = account_repo::read_by_id(ctx, id).await?;
//     let account: Arc<Account> = account_dto.into();
//     Ok(account)
// }

// pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<Arc<Account>, PersistenceError> {
//     match self.read_cache_by_name(name).await {
//         Some(account) => return Ok(account),
//         None => {
//             let account_dto = account_repo::read_by_name(name).await?;
//             let account: Arc<Account> = account_dto.into();
//             self.update_cache(account.clone()).await;
//             Ok(account)
//         }
//     }
// }
