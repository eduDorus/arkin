use std::sync::Arc;

use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;

use arkin_persistence::prelude::*;

use crate::{config::IngestorServiceConfig, IngestorFactory};

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
        let task_tracker = TaskTracker::new();
        let shutdown = CancellationToken::new();
        let ingestors = IngestorFactory::from_config(&self.config.ingestors, Arc::clone(&self.persistence_service));

        let mut tasks = Vec::with_capacity(ingestors.len());
        for ingestor in ingestors {
            // Move the ingestor into the async block
            let task_tracker = task_tracker.clone();
            let shutdown = shutdown.clone();
            let task = tokio::spawn(async move { ingestor.start(task_tracker, shutdown).await });
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
