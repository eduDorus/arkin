use std::sync::Arc;

use arkin_core::{Account, AccountListQuery, AccountQuery};
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::account_repo, stores::venue_store};

/// Convert AccountDTO to Arc<Account> by loading related entities
async fn dto_to_account(ctx: &PersistenceContext, dto: &account_repo::AccountDTO) -> Result<Arc<Account>, PersistenceError> {
    let venue = venue_store::read_by_id(ctx, &dto.venue_id).await?;

    let account = Account::builder()
        .id(dto.id)
        .venue(venue)
        .owner(dto.owner)
        .account_type(dto.account_type)
        .created(dto.created.to_utc())
        .updated(dto.updated.to_utc())
        .build();

    Ok(Arc::new(account))
}

async fn update_cache(ctx: &PersistenceContext, account: Arc<Account>) {
    // Check if we already have this account cached
    if ctx.cache.account_id.get(&account.id).await.is_some() {
        return;
    }

    ctx.cache.account_id.insert(account.id, account).await;
}

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

// pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Account>, PersistenceError> {
//     let account_dto = account_repo::read_by_id(ctx, id).await?;
//     let account: Arc<Account> = account_dto.into();
//     Ok(account)
// }

/// Load all accounts from database
pub async fn load_accounts(ctx: &PersistenceContext) -> Result<Vec<Arc<Account>>, PersistenceError> {
    let account_dtos = account_repo::list_all(ctx).await?;
    let mut accounts = Vec::with_capacity(account_dtos.len());

    for dto in account_dtos {
        match dto_to_account(ctx, &dto).await {
            Ok(account) => {
                update_cache(ctx, account.clone()).await;
                accounts.push(account);
            }
            Err(e) => {
                tracing::warn!("Failed to load account {}: {}", dto.id, e);
            }
        }
    }

    Ok(accounts)
}

/// Query accounts with in-memory filtering
pub async fn query(ctx: &PersistenceContext, query: &AccountQuery) -> Result<Arc<Account>, PersistenceError> {
    // For now, load all and filter, since accounts are few
    let all_accounts = load_accounts(ctx).await?;
    for account in all_accounts {
        if query.matches(&account) {
            return Ok(account);
        }
    }
    Err(PersistenceError::NotFound)
}

/// Query accounts with in-memory filtering
pub async fn query_list(ctx: &PersistenceContext, query: &AccountListQuery) -> Result<Vec<Arc<Account>>, PersistenceError> {
    let all_accounts = load_accounts(ctx).await?;

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_accounts);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Account>> = all_accounts.into_iter().filter(|account| query.matches(account)).collect();

    Ok(filtered)
}
//             let account_dto = account_repo::read_by_name(name).await?;
//             let account: Arc<Account> = account_dto.into();
//             self.update_cache(account.clone()).await;
//             Ok(account)
//         }
//     }
// }
