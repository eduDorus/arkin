use arkin_core::prelude::Venue;
use sqlx::prelude::*;
use uuid::Uuid;

#[derive(FromRow)]
pub struct DBVenue {
    pub id: Uuid,
    pub name: String,
    pub venue_type: String,
}

impl From<Venue> for DBVenue {
    fn from(venue: Venue) -> Self {
        Self {
            id: venue.id,
            name: venue.name,
            venue_type: venue.venue_type,
        }
    }
}
