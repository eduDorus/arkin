use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::PersistenceError;

#[derive(FromRow)]
pub struct AccountDTO {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub asset_id: Uuid,
    pub venue_id: Uuid,
    pub account_type: AccountType,
}

impl From<Arc<Account>> for AccountDTO {
    fn from(account: Arc<Account>) -> Self {
        Self {
            id: account.id,
            asset_id: account.asset.id(),
            venue_id: account.venue.id,
            account_type: account.account_type,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct AccountRepo {
    pool: PgPool,
}

impl AccountRepo {
    pub async fn insert(&self, account: AccountDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO accounts 
            (
                id, 
                name, 
                description,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            account.id,
            account.name,
            account.description,
            account.created_at,
            account.updated_at,
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
                name, 
                description,
                created_at,
                updated_at
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

    pub async fn read_by_name(&self, name: &str) -> Result<AccountDTO, PersistenceError> {
        let account = sqlx::query_as!(
            AccountDTO,
            r#"
            SELECT
                id, 
                name, 
                description,
                created_at,
                updated_at
            FROM accounts
            WHERE name = $1
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await?;

        match account {
            Some(account) => Ok(account),
            None => Err(PersistenceError::NotFound),
        }
    }
}
