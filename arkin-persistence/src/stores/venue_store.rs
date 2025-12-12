use std::sync::Arc;

use arkin_core::{Venue, VenueListQuery, VenueName, VenueQuery};
use uuid::Uuid;

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

/// Load all venues from database into cache
/// This should be called once during persistence initialization
pub async fn load_venues(ctx: &PersistenceContext) -> Result<Vec<Arc<Venue>>, PersistenceError> {
    let venue_dtos = venues_repo::list_all(ctx).await?;
    let mut venues = Vec::with_capacity(venue_dtos.len());

    for dto in venue_dtos {
        let venue: Arc<Venue> = Arc::new(dto.into());
        update_venue_cache(ctx, venue.clone()).await;
        venues.push(venue);
    }

    Ok(venues)
}

/// Query venues with in-memory filtering from cache
/// Assumes cache is already populated via load_venues()
pub async fn query(ctx: &PersistenceContext, query: &VenueQuery) -> Result<Arc<Venue>, PersistenceError> {
    // Try to read from cache first
    if let Some(name) = &query.name {
        if let Some(venue) = read_venue_name_cache(ctx, name).await {
            if query.matches(&venue) {
                return Ok(venue);
            }
        }
    }

    // Fallback to database
    if let Some(name) = &query.name {
        let venue_dto = venues_repo::read_by_name(ctx, name).await?;
        let venue: Arc<Venue> = Arc::new(venue_dto.into());
        update_venue_cache(ctx, venue.clone()).await;
        Ok(venue)
    } else {
        Err(PersistenceError::NotFound)
    }
}

/// Query venues with in-memory filtering from cache
/// Assumes cache is already populated via load_venues()
pub async fn query_list(ctx: &PersistenceContext, query: &VenueListQuery) -> Result<Vec<Arc<Venue>>, PersistenceError> {
    // Get all cached venues by iterating over the cache
    let all_venues: Vec<Arc<Venue>> = ctx.cache.venue_id.iter().map(|(_, venue)| venue).collect();

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_venues);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Venue>> = all_venues.into_iter().filter(|venue| query.matches(venue)).collect();

    Ok(filtered)
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
    match read_venue_name_cache(ctx, &name).await {
        Some(venue) => Ok(venue),
        None => {
            let venue = venues_repo::read_by_name(ctx, name).await?;
            let venue: Arc<Venue> = Arc::new(venue.into());
            update_venue_cache(ctx, venue.clone()).await;
            Ok(venue)
        }
    }
}
