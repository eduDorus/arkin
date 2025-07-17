use std::sync::Arc;

use uuid::Uuid;

use arkin_core::Venue;

use crate::{context::PersistenceContext, repos::pg::venues_repo, PersistenceError};

async fn update_venue_cache(ctx: &PersistenceContext, venue: Arc<Venue>) {
    ctx.cache.venue_id.insert(venue.id, venue).await;
}

async fn read_venue_cache(ctx: &PersistenceContext, id: &Uuid) -> Option<Arc<Venue>> {
    ctx.cache.venue_id.get(id).await
}

pub async fn insert(ctx: &PersistenceContext, venue: Arc<Venue>) -> Result<(), PersistenceError> {
    update_venue_cache(ctx, venue.clone()).await;
    venues_repo::insert(ctx, venue.into()).await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Venue>, PersistenceError> {
    match read_venue_cache(ctx, id).await {
        Some(venue) => return Ok(venue),
        None => {
            let venue = venues_repo::read_by_id(ctx, id).await?;
            let venue: Arc<Venue> = Arc::new(venue.into());
            update_venue_cache(ctx, venue.clone()).await;
            Ok(venue)
        }
    }
}
