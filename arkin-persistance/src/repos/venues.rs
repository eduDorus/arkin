use arkin_core::prelude::Venue;
use sqlx::{prelude::*, Error, PgPool};
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

impl From<DBVenue> for Venue {
    fn from(venue: DBVenue) -> Self {
        Self {
            id: venue.id,
            name: venue.name,
            venue_type: venue.venue_type,
        }
    }
}

#[derive(Debug)]
pub struct VenueRepo {
    pool: PgPool,
}

impl VenueRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, venue: Venue) -> Result<(), Error> {
        let venue = DBVenue::from(venue);
        sqlx::query!(
            r#"
            INSERT INTO venues (id, name, venue_type)
            VALUES ($1, $2, $3)
            "#,
            venue.id,
            venue.name,
            venue.venue_type,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Option<DBVenue>, Error> {
        let venue = sqlx::query_as!(
            DBVenue,
            r#"
            SELECT * FROM venues
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(venue)
    }
}
