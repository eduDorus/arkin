use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{EventPayload, PersistenceReader, VenueQuery};

use super::Venue;

#[derive(Debug, Clone, Copy, PartialEq, Type, Display, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "account_owner", rename_all = "snake_case")]
pub enum AccountOwner {
    User,
    Venue,
}

#[derive(Debug, Clone, Copy, PartialEq, Type, Display, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "account_type", rename_all = "snake_case")]
pub enum AccountType {
    Spot,
    Margin,
    Equity,
}

#[derive(Debug, Clone, TypedBuilder, Hash)]
pub struct Account {
    pub id: Uuid,
    pub venue: Arc<Venue>,
    pub owner: AccountOwner,
    pub account_type: AccountType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl Account {
    pub fn is_user_account(&self) -> bool {
        self.owner == AccountOwner::User
    }

    pub fn is_venue_account(&self) -> bool {
        self.owner == AccountOwner::Venue
    }

    pub fn is_spot_account(&self) -> bool {
        self.account_type == AccountType::Spot
    }

    pub fn is_margin_account(&self) -> bool {
        self.account_type == AccountType::Margin
    }

    pub fn is_equity_account(&self) -> bool {
        self.account_type == AccountType::Equity
    }

    pub fn is_venue(&self, venue: &Arc<Venue>) -> bool {
        self.venue == *venue
    }
}

#[async_trait]
impl EventPayload for Account {
    type Dto = AccountDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let venue = persistence
            .get_venue(&VenueQuery::builder().id(dto.venue_id).build())
            .await
            .context(format!("Failed to get venue with id {}", dto.venue_id))?;

        Ok(Account::builder()
            .id(dto.id)
            .venue(venue)
            .owner(dto.owner)
            .account_type(dto.account_type)
            .created(dto.created)
            .updated(dto.updated)
            .build())
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Account {}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{} ({})", self.owner, self.venue, self.account_type)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountDto {
    pub id: Uuid,
    pub venue_id: Uuid,
    pub owner: AccountOwner,
    pub account_type: AccountType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl From<Account> for AccountDto {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            venue_id: account.venue.id,
            owner: account.owner,
            account_type: account.account_type,
            created: account.created,
            updated: account.updated,
        }
    }
}
