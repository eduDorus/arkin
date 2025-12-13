use std::sync::Arc;

use arkin_core::VenueOrder;

use arkin_core::PersistenceError;

use crate::repos::ch::venue_order_repo::VenueOrderClickhouseDTO;
use crate::{context::PersistenceContext, repos::ch::venue_order_repo};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    venue_order_repo::create_table(ctx).await
}

pub async fn insert(ctx: &PersistenceContext, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
    venue_order_repo::insert(ctx, VenueOrderClickhouseDTO::from_model(&order, ctx.instance.id)).await
}

pub async fn insert_batch(ctx: &PersistenceContext, orders: &[Arc<VenueOrder>]) -> Result<(), PersistenceError> {
    let dtos: Vec<_> = orders
        .iter()
        .map(|o| VenueOrderClickhouseDTO::from_model(o, ctx.instance.id))
        .collect();
    venue_order_repo::insert_batch(ctx, &dtos).await
}

// pub async fn insert(ctx: &PersistenceContext, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
//     venue_order_repo::insert(ctx, order.into()).await
// }

// pub async fn update(ctx: &PersistenceContext, order: Arc<VenueOrder>) -> Result<(), PersistenceError> {
//     venue_order_repo::update(ctx, order.into()).await
// }
