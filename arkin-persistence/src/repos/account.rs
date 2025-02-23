use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(FromRow)]
pub struct AccountDTO {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub venue_id: Uuid,
    pub owner: AccountOwner,
    pub account_type: AccountType,
}

impl From<Arc<Account>> for AccountDTO {
    fn from(account: Arc<Account>) -> Self {
        Self {
            id: account.id,
            asset_id: account.asset.id(),
            venue_id: account.venue.id,
            owner: account.owner.clone(),
            account_type: account.account_type.clone(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct AccountRepo {
    pool: PgPool,
    instance: Arc<Instance>,
}

impl AccountRepo {
    pub async fn insert(&self, account: AccountDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO accounts 
            (
                id, 
                instance_id,
                asset_id, 
                venue_id,
                owner,
                account_type
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            account.id,
            self.instance.id,
            account.asset_id,
            account.venue_id,
            account.owner as AccountOwner,
            account.account_type as AccountType,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<AccountDTO, PersistenceError> {
        let account = sqlx::query_as!(
            AccountDTO,
            r#"
            SELECT
                id, 
                asset_id, 
                venue_id,
                owner AS "owner:AccountOwner",
                account_type AS "account_type:AccountType"
            FROM accounts
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match account {
            Some(account) => Ok(account),
            None => Err(PersistenceError::NotFound),
        }
    }

    pub async fn read_by_instance(&self) -> Result<AccountDTO, PersistenceError> {
        let account = sqlx::query_as!(
            AccountDTO,
            r#"
            SELECT
                id, 
                asset_id, 
                venue_id,
                owner AS "owner:AccountOwner",
                account_type AS "account_type:AccountType"
            FROM accounts
            WHERE instance_id = $1
            "#,
            self.instance.id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match account {
            Some(account) => Ok(account),
            None => Err(PersistenceError::NotFound),
        }
    }
}
