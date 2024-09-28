use std::sync::Arc;

use async_trait::async_trait;

use arkin_persistance::prelude::*;

use crate::{config::IngestorServiceConfig, IngestorFactory};

#[async_trait]
pub trait Ingestor {
    async fn start(&self);
}

pub struct IngestorService {
    ingestors: Vec<Box<dyn Ingestor>>,
}

impl IngestorService {
    pub fn from_config(config: &IngestorServiceConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            ingestors: IngestorFactory::from_config(&config.ingestors, persistance_service),
        }
    }

    pub async fn start(&self) {
        // Probably want to spawn these tasks
        for ingestor in &self.ingestors {
            ingestor.start().await;
        }
    }
}
