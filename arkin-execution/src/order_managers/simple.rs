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

        let mut rx = self.pubsub.subscribe();

        loop {
            tokio::select! {
              Ok(event) = rx.recv() => {
                match event {
                  Event::ExecutionOrderNew(order) => {
                    info!("SimpleOrderManager received order: {}", order);
                    let venue_order = Arc::new(VenueOrder::builder()
                        .id(order.id)
                        .portfolio(test_portfolio())
                        .instrument(order.instrument.to_owned())
                        .side(order.side)
                        .order_type(order.order_type.into())
                        .price(order.price)
                        .quantity(order.quantity)
                        .build());

                    self.pubsub.publish(venue_order).await;
                  }
                  Event::VenueOrderUpdate(order) => {
                    info!("SimpleOrderManager received order update: {}", order);
                    // if let Err(e) = self.order_update(fill.clone()).await {
                    //     error!("Failed to process fill: {}", e);
                    // }
                  }
                  _ => {}
                }
              }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
