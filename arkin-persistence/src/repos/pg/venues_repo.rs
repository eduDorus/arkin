use std::sync::Arc;

use sqlx::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use arkin_core::{Venue, VenueName, VenueType};

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

#[derive(FromRow)]
pub struct VenueDTO {
    pub id: Uuid,
    pub name: String,
    pub venue_type: VenueType,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<Arc<Venue>> for VenueDTO {
    fn from(venue: Arc<Venue>) -> Self {
        Self {
            id: venue.id,
            name: venue.name.to_string(),
            venue_type: venue.venue_type.clone(),
            created: venue.created.into(),
            updated: venue.updated.into(),
        }
    }
}

impl From<VenueDTO> for Venue {
    fn from(venue: VenueDTO) -> Self {
        Self {
            id: venue.id,
            name: venue.name.parse().expect("Invalid VenueName"),
            venue_type: venue.venue_type,
            created: venue.created.into(),
            updated: venue.updated.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, venue: VenueDTO) -> Result<(), PersistenceError> {
    sqlx::query!(
        r#"
            INSERT INTO venues 
            (
                id, 
                name, 
                venue_type,
                created,
                updated
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        venue.id,
        venue.name,
        venue.venue_type as VenueType,
        venue.created,
        venue.updated
    )
    .execute(&ctx.pg_pool)
    .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<VenueDTO, PersistenceError> {
    let id = sqlx::query_as!(
        VenueDTO,
        r#"
            SELECT 
                id,
                name,
                venue_type AS "venue_type:VenueType",
                created,
                updated
            FROM venues
            WHERE id = $1
            "#,
        id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match id {
        Some(id) => Ok(id),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &VenueName) -> Result<VenueDTO, PersistenceError> {
    let id = sqlx::query_as!(
        VenueDTO,
        r#"
            SELECT 
                id,
                name,
                venue_type AS "venue_type:VenueType",
                created,
                updated
            FROM venues
            WHERE name = $1
            "#,
        name.to_string(),
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match id {
        Some(id) => Ok(id),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn list_all(ctx: &PersistenceContext) -> Result<Vec<VenueDTO>, PersistenceError> {
    let venues = sqlx::query_as!(
        VenueDTO,
        r#"
            SELECT 
                id,
                name,
                venue_type AS "venue_type:VenueType",
                created,
                updated
            FROM venues
            ORDER BY name
            "#,
    )
    .fetch_all(&ctx.pg_pool)
    .await?;

    Ok(venues)
}
