use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;
use tracing::{debug, error};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{repos::InsightsClickhouseRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]

pub struct InsightsStore {
    insights_repo: InsightsClickhouseRepo,
    #[builder(default)]
    insights_buffer: Arc<Mutex<Vec<Arc<Insight>>>>,
    buffer_size: usize,
    #[builder(default)]
    flush_tracker: TaskTracker,
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
            debug!("No insights to flush.");
            return Ok(());
        }

        let repo = self.insights_repo.clone();
        let insights = insights.into_iter().map(|t| t.into()).collect::<Vec<_>>();

        self.flush_tracker.spawn(async move {
            debug!("Flushing {} insights", insights.len());

            // Insert the insights into the database
            loop {
                match repo.insert_batch(&insights).await {
                    Ok(_) => {
                        debug!("Successfully flushed {} insights", insights.len());
                        break;
                    }
                    Err(e) => {
                        error!("Failed to flush insights: {}", e);
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

    pub async fn close(&self) -> Result<(), PersistenceError> {
        self.flush_tracker.close();
        self.flush_tracker.wait().await;
        self.insights_repo.close().await
    }

    pub async fn insert_buffered(&self, insight: Arc<Insight>) -> Result<(), PersistenceError> {
        if !insight.persist {
            return Ok(());
        }

        let mut lock = self.insights_buffer.lock().await; // Wait for lock
        lock.push(insight);
        Ok(())
    }

    pub async fn insert_buffered_vec(&self, insights: Vec<Arc<Insight>>) -> Result<(), PersistenceError> {
        // Filter out any insights that don't need to be persisted
        let insights = insights.into_iter().filter(|i| i.persist).collect::<Vec<_>>();
        if insights.is_empty() {
            return Ok(());
        }

        let mut lock = self.insights_buffer.lock().await; // Wait for lock
        lock.extend(insights);
        Ok(())
    }
}
