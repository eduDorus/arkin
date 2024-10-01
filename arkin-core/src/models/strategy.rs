use std::fmt;

use uuid::Uuid;

use crate::{
    events::{EventType, EventTypeOf},
    Event,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Strategy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

impl Strategy {
    pub fn new(name: String, description: String) -> Self {
        Strategy {
            id: Uuid::new_v4(),
            name,
            description,
        }
    }
}

impl EventTypeOf for Strategy {
    fn event_type() -> EventType {
        EventType::Strategy
    }
}

impl TryFrom<Event> for Strategy {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Strategy(strategy) = event {
            Ok(strategy)
        } else {
            Err(())
        }
    }
}

impl From<Strategy> for Event {
    fn from(strategy: Strategy) -> Self {
        Event::Strategy(strategy)
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
