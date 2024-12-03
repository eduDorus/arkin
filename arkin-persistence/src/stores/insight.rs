use std::sync::Arc;

use derive_builder::Builder;
use tokio::sync::Mutex;
use tracing::error;

use arkin_core::prelude::*;

use crate::{repos::InsightsRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct InsightsStore {
    insights_repo: InsightsRepo,
    #[builder(default)]
    insights_buffer: Arc<Mutex<Vec<Arc<Insight>>>>,
    buffer_size: usize,
}

impl InsightsStore {
    pub async fn flush(&self) -> Result<(), PersistenceError> {
        // Lock and extract ticks without cloning
        let insights = {
            let mut lock = self.insights_buffer.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        let insights = insights.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if let Err(e) = self.insights_repo.insert_batch(insights).await {
            error!("Failed to flush ticks: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<(), PersistenceError> {
        let should_commit = {
            let lock = self.insights_buffer.lock().await;
            lock.len() >= self.buffer_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, insight: Arc<Insight>) -> Result<(), PersistenceError> {
        self.insights_repo.insert(insight.into()).await
    }

    pub async fn insert_buffered(&self, insight: Arc<Insight>) -> Result<(), PersistenceError> {
        {
            let mut lock = self.insights_buffer.lock().await; // Wait for lock
            lock.push(insight);
        }

        self.commit().await
    }

    pub async fn insert_buffered_vec(&self, insights: Vec<Arc<Insight>>) -> Result<(), PersistenceError> {
        {
            let mut lock = self.insights_buffer.lock().await; // Wait for lock
            lock.extend(insights);
        }

        self.commit().await
    }
}
