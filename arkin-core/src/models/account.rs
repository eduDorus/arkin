use std::{fmt, sync::Arc};

use sqlx::Type;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::Event;

use super::{Tradable, Venue};

#[derive(Debug, Clone, PartialEq, Type)]
#[sqlx(type_name = "account_owner", rename_all = "snake_case")]
pub enum AccountOwner {
    User,
    Venue,
}

impl fmt::Display for AccountOwner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountOwner::User => write!(f, "U"),
            AccountOwner::Venue => write!(f, "V"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Type)]
#[sqlx(type_name = "account_type", rename_all = "snake_case")]
pub enum AccountType {
    Spot,
    Margin,
    Instrument,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Spot => write!(f, "S"),
            AccountType::Margin => write!(f, "M"),
            AccountType::Instrument => write!(f, "I"),
        }
    }
}

/// Each account references a specific currency.
/// We'll store the balance as a Decimal, but you could use integer
/// amounts of "cents" or "atomic units" for real usage.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Account {
    pub id: Uuid,
    pub asset: Tradable,
    pub venue: Arc<Venue>,
    pub owner: AccountOwner,
    pub account_type: AccountType,
}

impl Account {
    pub fn is_user_account(&self) -> bool {
        match self.owner {
            AccountOwner::User => true,
            _ => false,
        }
    }

    pub fn is_venue_account(&self) -> bool {
        match self.owner {
            AccountOwner::Venue => true,
            _ => false,
        }
    }

    pub fn is_spot_account(&self) -> bool {
        match self.account_type {
            AccountType::Spot => true,
            _ => false,
        }
    }

    pub fn is_margin_account(&self) -> bool {
        match self.account_type {
            AccountType::Margin => true,
            _ => false,
        }
    }

    pub fn is_venue(&self, venue: &Arc<Venue>) -> bool {
        self.venue == *venue
    }

    pub fn has_asset(&self, asset: &Tradable) -> bool {
        self.asset == *asset
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}_{}", self.account_type, self.venue, self.asset)
    }
}

impl From<Arc<Account>> for Event {
    fn from(account: Arc<Account>) -> Self {
        Event::AccountNew(account)
    }
}
