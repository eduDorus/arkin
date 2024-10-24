use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use arkin_persistance::prelude::*;
use tracing::error;

use crate::{config::IngestorServiceConfig, IngestorFactory};

#[async_trait]
pub trait Ingestor: Send + Sync {
    async fn start(&self) -> Result<()>;
}

pub struct IngestorService {
    config: IngestorServiceConfig,
    persistance_service: Arc<PersistanceService>,
}

impl IngestorService {
    pub fn from_config(config: &IngestorServiceConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            config: config.clone(),
            persistance_service,
        }
    }

    pub async fn start(&self) {
        let ingestors = IngestorFactory::from_config(&self.config.ingestors, Arc::clone(&self.persistance_service));

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
