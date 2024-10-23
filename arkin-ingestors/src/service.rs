use std::sync::Arc;

use async_trait::async_trait;

use arkin_persistance::prelude::*;

use crate::{config::IngestorServiceConfig, IngestorFactory};

#[async_trait]
pub trait Ingestor: Send + Sync {
    async fn start(&self);
}

pub struct IngestorService {
    ingestors: Vec<Arc<Box<dyn Ingestor>>>,
}

impl IngestorService {
    pub fn from_config(config: &IngestorServiceConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            ingestors: IngestorFactory::from_config(&config.ingestors, persistance_service),
        }
    }

    pub async fn start(&self) {
        let mut tasks = vec![];

        for ingestor in &self.ingestors {
            let ingestor_clone = Arc::clone(ingestor); // Clone the Arc for sharing
            let task = tokio::spawn(async move {
                ingestor_clone.start().await;
            });
            tasks.push(task);
        }

        // Optionally, wait for all tasks to complete
        for task in tasks {
            let _ = task.await; // Handle errors as needed
        }
    }
}
