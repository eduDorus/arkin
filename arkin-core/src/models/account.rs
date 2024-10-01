use std::fmt;

use rust_decimal::prelude::*;
use uuid::Uuid;

use crate::{
    events::{EventType, EventTypeOf},
    Event,
};

use super::Venue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account {
    pub id: Uuid,
    pub venue: Venue,
    pub name: String,
    pub balance: Decimal,
}

impl Account {
    pub fn new(venue: Venue, name: String, balance: Decimal) -> Self {
        Account {
            id: Uuid::new_v4(),
            venue,
            name,
            balance,
        }
    }
}

impl EventTypeOf for Account {
    fn event_type() -> EventType {
        EventType::Account
    }
}

impl TryFrom<Event> for Account {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Account(account) = event {
            Ok(account)
        } else {
            Err(())
        }
    }
}

impl From<Account> for Event {
    fn from(account: Account) -> Self {
        Event::Account(account)
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
