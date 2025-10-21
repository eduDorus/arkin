use std::sync::Arc;

use arkin_core::VenueName;
use uuid::Uuid;

use arkin_core::Venue;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::venues_repo};

async fn update_venue_cache(ctx: &PersistenceContext, venue: Arc<Venue>) {
    ctx.cache.venue_id.insert(venue.id, venue.clone()).await;
    ctx.cache.venue_name.insert(venue.name, venue).await;
}

async fn read_venue_id_cache(ctx: &PersistenceContext, id: &Uuid) -> Option<Arc<Venue>> {
    ctx.cache.venue_id.get(id).await
}

async fn read_venue_name_cache(ctx: &PersistenceContext, name: &VenueName) -> Option<Arc<Venue>> {
    ctx.cache.venue_name.get(name).await
}

pub async fn insert(ctx: &PersistenceContext, venue: Arc<Venue>) -> Result<(), PersistenceError> {
    update_venue_cache(ctx, venue.clone()).await;
    venues_repo::insert(ctx, venue.into()).await
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Venue>, PersistenceError> {
    match read_venue_id_cache(ctx, id).await {
        Some(venue) => Ok(venue),
        None => {
            let venue = venues_repo::read_by_id(ctx, id).await?;
            let venue: Arc<Venue> = Arc::new(venue.into());
            update_venue_cache(ctx, venue.clone()).await;
            Ok(venue)
        }
    }
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &VenueName) -> Result<Arc<Venue>, PersistenceError> {
    match read_venue_name_cache(ctx, name).await {
        Some(venue) => Ok(venue),
        None => {
            let venue = venues_repo::read_by_name(ctx, name).await?;
            let venue: Arc<Venue> = Arc::new(venue.into());
            update_venue_cache(ctx, venue.clone()).await;
            Ok(venue)
        }
    }
}
