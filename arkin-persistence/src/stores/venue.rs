use std::sync::Arc;

use derive_builder::Builder;
use moka2::future::Cache;
use uuid::Uuid;

use arkin_core::Venue;

use crate::{repos::VenueRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
pub struct VenueStore {
    venue_repo: VenueRepo,
    #[builder(default = "Cache::new(1000)")]
    venue_cache: Cache<Uuid, Arc<Venue>>,
}

impl VenueStore {
    async fn update_venue_cache(&self, venue: Arc<Venue>) {
        self.venue_cache.insert(venue.id, venue).await;
    }

    async fn read_venue_cache(&self, id: &Uuid) -> Option<Arc<Venue>> {
        self.venue_cache.get(id).await
    }

    pub async fn insert(&self, venue: Arc<Venue>) -> Result<(), PersistenceError> {
        self.update_venue_cache(venue.clone()).await;
        self.venue_repo.insert(venue.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Venue>, PersistenceError> {
        match self.read_venue_cache(id).await {
            Some(venue) => return Ok(venue),
            None => {
                let venue = self.venue_repo.read_by_id(id).await?;
                let venue: Arc<Venue> = Arc::new(venue.into());
                self.update_venue_cache(venue.clone()).await;
                Ok(venue)
            }
        }
    }
}
