use std::sync::Arc;

use sqlx::{prelude::*, PgPool};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::{Venue, VenueType};

use crate::PersistenceError;

#[derive(FromRow)]
pub struct VenueDTO {
    pub id: Uuid,
    pub name: String,
    pub venue_type: VenueType,
}

impl From<Arc<Venue>> for VenueDTO {
    fn from(venue: Arc<Venue>) -> Self {
        Self {
            id: venue.id,
            name: venue.name.clone(),
            venue_type: venue.venue_type.clone(),
        }
    }
}

impl From<VenueDTO> for Venue {
    fn from(venue: VenueDTO) -> Self {
        Self {
            id: venue.id,
            name: venue.name,
            venue_type: venue.venue_type,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct VenueRepo {
    pool: PgPool,
}

impl VenueRepo {
    pub async fn insert(&self, venue: VenueDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO venues 
            (
                id, 
                name, 
                venue_type
            ) VALUES ($1, $2, $3)
            "#,
            venue.id,
            venue.name,
            venue.venue_type as VenueType,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<VenueDTO, PersistenceError> {
        let id = sqlx::query_as!(
            VenueDTO,
            r#"
            SELECT 
                id,
                name,
                venue_type AS "venue_type:VenueType"
            FROM venues
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        match id {
            Some(id) => Ok(id),
            None => Err(PersistenceError::NotFound),
        }
    }
}
