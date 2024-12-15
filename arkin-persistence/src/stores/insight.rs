use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{repos::InsightsRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

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

        // If there are no insights to flush, return early
        if insights.is_empty() {
            info!("No insights to flush.");
            return Ok(());
        }

        let repo = self.insights_repo.clone();
        let insights = insights.into_iter().map(|t| t.into()).collect::<Vec<_>>();

        tokio::spawn(async move {
            info!("Flushing {} insights", insights.len());

            // Insert the insights into the database
            loop {
                match repo.insert_batch(&insights).await {
                    Ok(_) => {
                        info!("Successfully flushed {} insights", insights.len());
                        break;
                    }
                    Err(_) => {
                        error!("Failed to flush insights, will try again in 5 seconds");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
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
        Ok(())
        // self.commit().await
    }
}
