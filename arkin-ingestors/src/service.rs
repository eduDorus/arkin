use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use arkin_persistence::prelude::*;
use tracing::error;

use crate::{config::IngestorServiceConfig, IngestorFactory};

#[async_trait]
pub trait Ingestor: Send + Sync {
    async fn start(&self) -> Result<()>;
}

pub struct IngestorService {
    config: IngestorServiceConfig,
    persistence_service: Arc<PersistenceService>,
}

impl IngestorService {
    pub fn from_config(config: &IngestorServiceConfig, persistence_service: Arc<PersistenceService>) -> Self {
        Self {
            config: config.clone(),
            persistence_service,
        }
    }

    pub async fn start(&self) {
        let ingestors = IngestorFactory::from_config(&self.config.ingestors, Arc::clone(&self.persistence_service));

        let mut tasks = Vec::with_capacity(ingestors.len());
        for ingestor in ingestors {
            // Move the ingestor into the async block
            let task = tokio::spawn(async move { ingestor.start().await });
            tasks.push(task);
        }

        // Optionally, wait for all tasks to complete
        for task in tasks {
            if let Err(e) = task.await {
                error!("Ingestor task failed: {}", e);
            }
        }
    }
}
