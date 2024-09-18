use std::fmt;

use uuid::Uuid;

use crate::{
    events::{EventType, EventTypeOf},
    Event,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Venue {
    pub id: Uuid,
    pub name: String,
    pub venue_type: String,
}

impl Venue {
    pub fn new(name: String, venue_type: String) -> Self {
        Venue {
            id: Uuid::new_v4(),
            name,
            venue_type,
        }
    }
}

impl EventTypeOf for Venue {
    fn event_type() -> EventType {
        EventType::Venue
    }
}

impl TryFrom<Event> for Venue {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Venue(venue) = event {
            Ok(venue)
        } else {
            Err(())
        }
    }
}

impl From<Venue> for Event {
    fn from(venue: Venue) -> Self {
        Event::Venue(venue)
    }
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
