use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerService};

#[derive(Debug, TypedBuilder)]
pub struct DefaultOrderManager {
    pubsub: Arc<PubSub>,
}

#[async_trait]
impl OrderManagerService for DefaultOrderManager {}

#[async_trait]
impl OrderManager for DefaultOrderManager {}

#[async_trait]
impl RunnableService for DefaultOrderManager {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting order manager...");

        let mut rx = self.pubsub.subscribe();

        loop {
            tokio::select! {
              Ok(event) = rx.recv() => {
                match event {
                  Event::ExecutionOrderNew(order) => {
                    info!("SimpleOrderManager received order: {}", order);
                  }
                  Event::VenueOrderUpdate(order) => {
                    info!("SimpleOrderManager received order update: {}", order);
                  }
                  _ => {}
                }
              }
                _ = shutdown.cancelled() => {
                    info!("Execution shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}
