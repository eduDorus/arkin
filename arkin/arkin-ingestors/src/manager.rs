use async_trait::async_trait;

use crate::{config::IngestorManagerConfig, IngestorFactory};

#[async_trait]
pub trait IngestorModule {
    async fn start(&self);
}

pub struct IngestorManager {
    ingestors: Vec<Box<dyn IngestorModule>>,
}

impl IngestorManager {
    pub fn from_config(config: &IngestorManagerConfig) -> Self {
        Self {
            ingestors: IngestorFactory::from_config(&config.ingestors),
        }
    }

    pub async fn start(&self) {
        // Probably want to spawn these tasks
        for ingestor in &self.ingestors {
            ingestor.start().await;
        }
    }
}
