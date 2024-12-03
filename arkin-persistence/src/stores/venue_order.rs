use std::sync::Arc;

use arkin_core::VenueOrder;
use derive_builder::Builder;

use crate::{repos::VenueOrderRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct VenueOrderStore {
    venue_order_repo: VenueOrderRepo,
}

impl VenueOrderStore {
    pub async fn insert(&self, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
        self.venue_order_repo.insert(order.into()).await
    }
}
