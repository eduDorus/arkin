use std::sync::Arc;

use arkin_core::VenueOrder;
use typed_builder::TypedBuilder;

use crate::{repos::VenueOrderRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct VenueOrderStore {
    venue_order_repo: VenueOrderRepo,
}

impl VenueOrderStore {
    pub async fn insert(&self, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
        self.venue_order_repo.insert(order.into()).await
    }

    pub async fn update(&self, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
        self.venue_order_repo.update(order.into()).await
    }
}
