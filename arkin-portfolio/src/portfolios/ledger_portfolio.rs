use std::sync::Arc;

use async_trait::async_trait;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::ledger::Ledger;

#[derive(Debug, TypedBuilder)]
pub struct LedgerPortfolio {
    pub pubsub: Arc<PubSub>,
    pub ledger: Ledger,
}

#[async_trait]
impl RunnableService for LedgerPortfolio {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        let mut rx = self.pubsub.subscribe();

        while !shutdown.is_cancelled() {
            select! {
                Ok(event) = rx.recv() => {
                  match event {
                    Event::VenueOrderUpdate(e) => {
                      info!("Venue order update: {:?}", e);
                      // self.ledger.update_order(e).await?;
                    },
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
