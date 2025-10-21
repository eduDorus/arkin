use std::sync::Arc;

use arkin_core::prelude::*;
use sqlx::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

#[derive(FromRow)]
pub struct AccountDTO {
    pub id: Uuid,
    pub venue_id: Uuid,
    pub owner: AccountOwner,
    pub account_type: AccountType,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<Arc<Account>> for AccountDTO {
    fn from(account: Arc<Account>) -> Self {
        Self {
            id: account.id,
            venue_id: account.venue.id,
            owner: account.owner,
            account_type: account.account_type,
            created: account.created.into(),
            updated: account.updated.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, account: AccountDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO accounts 
            (
                id, 
                instance_id,
                venue_id,
                owner,
                account_type,
                created,
                updated
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        account.id,
        ctx.instance.id,
        account.venue_id,
        account.owner as AccountOwner,
        account.account_type as AccountType,
        account.created,
        account.updated
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<AccountDTO, PersistenceError> {
    let account = sqlx::query_as!(
        AccountDTO,
        r#"
            SELECT
                id, 
                venue_id,
                owner AS "owner:AccountOwner",
                account_type AS "account_type:AccountType",
                created,
                updated
            FROM accounts
            WHERE id = $1
            "#,
        id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match account {
        Some(account) => Ok(account),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_instance(ctx: &PersistenceContext) -> Result<AccountDTO, PersistenceError> {
    let account = sqlx::query_as!(
        AccountDTO,
        r#"
            SELECT
                id, 
                venue_id,
                owner AS "owner:AccountOwner",
                account_type AS "account_type:AccountType",
                created,
                updated
            FROM accounts
            WHERE instance_id = $1
            "#,
        ctx.instance.id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match account {
        Some(account) => Ok(account),
        None => Err(PersistenceError::NotFound),
    }
}
