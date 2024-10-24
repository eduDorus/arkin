use anyhow::Result;
use arkin_core::prelude::Venue;
use uuid::Uuid;

use crate::repos::VenueRepo;

#[derive(Debug)]
pub struct VenueService {
    venue_repo: VenueRepo,
}

impl VenueService {
    pub fn new(venue_repo: VenueRepo) -> Self {
        Self { venue_repo }
    }

    pub async fn insert(&self, venue: Venue) -> Result<()> {
        self.venue_repo.create(venue.into()).await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: Uuid) -> Result<Option<Venue>> {
        let venue_repo = &self.venue_repo;

        let venue = venue_repo.read_by_id(id).await?;
        Ok(venue.map(|v| v.into()))
    }
}
