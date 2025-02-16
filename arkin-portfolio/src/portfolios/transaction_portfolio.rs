use std::sync::Arc;

use async_trait::async_trait;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

#[derive(Debug, TypedBuilder)]
pub struct TransactionPortfolio {
    pub pubsub: Arc<PubSub>,

    pub transactions: Vec<Arc<Transaction>>,
}

#[async_trait]
impl RunnableService for TransactionPortfolio {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        let mut rx = self.pubsub.subscribe();

        while !shutdown.is_cancelled() {
            select! {
                Ok(event) = rx.recv() => {
                  match event {
                    Event::VenueOrderUpdate(e) => info!("Venue order update: {:?}", e),
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
