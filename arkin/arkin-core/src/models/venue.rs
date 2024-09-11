use std::fmt;

use crate::{
    events::{EventType, EventTypeOf},
    Event,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Venue {
    pub id: u32,
    pub name: String,
    pub venue_type: String,
}

impl Venue {
    pub fn new(id: u32, name: String, venue_type: String) -> Self {
        Venue {
            id,
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
