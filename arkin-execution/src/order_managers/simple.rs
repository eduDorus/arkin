use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerError};

#[derive(Debug, TypedBuilder)]
pub struct SimpleOrderManager {
    pubsub: Arc<PubSub>,
}

#[async_trait]
impl OrderManager for SimpleOrderManager {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        let mut execution_orders = self.pubsub.subscribe::<ExecutionOrder>();
        let mut venue_order_updates = self.pubsub.subscribe::<VenueOrderUpdate>();
        loop {
            tokio::select! {
                Ok(order) = execution_orders.recv() => {
                    info!("SimpleOrderManager received order: {}", order);
                    let venue_order = VenueOrder::builder()
                        .id(order.id)
                        .portfolio(test_portfolio())
                        .instrument(order.instrument.to_owned())
                        .side(order.side)
                        .order_type(order.order_type.into())
                        .price(order.price)
                        .quantity(order.quantity)
                        .build();

                    self.pubsub.publish::<VenueOrder>(venue_order.into());
                }
                Ok(order) = venue_order_updates.recv() => {
                    info!("SimpleOrderManager received order update: {}", order);
                    // if let Err(e) = self.order_update(fill.clone()).await {
                    //     error!("Failed to process fill: {}", e);
                    // }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
