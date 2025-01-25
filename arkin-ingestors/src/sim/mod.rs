use std::sync::Arc;

use arkin_core::Instrument;
use arkin_persistence::PersistenceService;
use async_trait::async_trait;
use futures_util::StreamExt;
use time::OffsetDateTime;
use tokio::pin;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use crate::{Ingestor, IngestorError};

#[derive(Debug, TypedBuilder)]
pub struct SimIngestor {
    // pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    instruments: Vec<Arc<Instrument>>,
    start: OffsetDateTime,
    end: OffsetDateTime,
}

#[async_trait]
impl Ingestor for SimIngestor {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), IngestorError> {
        info!("Starting SimIngestor...");

        let stream = self
            .persistence
            .tick_store
            .stream_range(&self.instruments, self.start, self.end)
            .await?;

        pin!(stream);

        let mut counter = 0;
        while let Some(data) = stream.next().await {
            match data {
                Ok(_trade) => {
                    counter += 1;
                }
                Err(e) => {
                    return Err(IngestorError::from(e));
                }
            }
        }
        info!("Counted: {}", counter);

        Ok(())
    }
}
