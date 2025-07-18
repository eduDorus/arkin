use std::{fmt, sync::Arc};

use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use super::Venue;

#[derive(Debug, Clone, Copy, PartialEq, Type, Display, Hash)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "account_owner", rename_all = "snake_case")]
pub enum AccountOwner {
    User,
    Venue,
}

#[derive(Debug, Clone, Copy, PartialEq, Type, Display, Hash)]
#[strum(serialize_all = "snake_case")]
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

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Account {}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}_{}", self.owner, self.venue, self.account_type)
    }
}
