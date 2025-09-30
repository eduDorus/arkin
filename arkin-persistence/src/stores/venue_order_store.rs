use std::sync::Arc;

use arkin_core::VenueOrder;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::venue_order_repo};

pub async fn insert(ctx: &PersistenceContext, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
    venue_order_repo::insert(ctx, order.into()).await
}

pub async fn update(ctx: &PersistenceContext, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
    venue_order_repo::update(ctx, order.into()).await
}
