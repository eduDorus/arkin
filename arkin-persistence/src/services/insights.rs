use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;
use tokio::sync::Mutex;
use tracing::error;

use crate::repos::InsightsRepo;

#[derive(Debug)]
pub struct InsightsService {
    insights_repo: Arc<InsightsRepo>,
    insights_batch: Mutex<Vec<Insight>>,
    batch_size: usize,
}

impl InsightsService {
    pub fn new(insights_repo: Arc<InsightsRepo>, batch_size: usize) -> Self {
        Self {
            insights_repo,
            insights_batch: Mutex::new(Vec::new()),
            batch_size,
        }
    }

    pub async fn flush(&self) -> Result<()> {
        // Lock and extract ticks without cloning
        let ticks = {
            let mut lock = self.insights_batch.lock().await;
            std::mem::take(&mut *lock) // Take ownership and clear the vector
        };

        if let Err(e) = self.insights_repo.insert_batch(ticks).await {
            error!("Failed to flush ticks: {}", e);
            return Err(e);
        }
        Ok(())
    }

    pub async fn commit(&self) -> Result<()> {
        let should_commit = {
            let lock = self.insights_batch.lock().await;
            lock.len() >= self.batch_size
        };

        if should_commit {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn insert(&self, insight: Insight) -> Result<()> {
        self.insights_repo.insert(insight).await
    }

    pub async fn insert_batch(&self, insight: Insight) -> Result<()> {
        {
            let mut lock = self.insights_batch.lock().await; // Wait for lock
            lock.push(insight);
        }

        self.commit().await?;
        Ok(())
    }

    pub async fn insert_batch_vec(&self, insights: Vec<Insight>) -> Result<()> {
        {
            let mut lock = self.insights_batch.lock().await; // Wait for lock
            lock.extend(insights);
        }

        self.commit().await?;
        Ok(())
    }

    // I DON'T THINK WE WILL EVER READ INSIGHTS INTO OUR SYSTEM
    // pub async fn read_range_by_instrument_id_and_feature_id(
    //     &self,
    //     instrument_id: Uuid,
    //     feature_id: &str,
    //     from: OffsetDateTime,
    //     to: OffsetDateTime,
    // ) -> Result<Vec<Insight>> {
    //     // Load insights
    //     let insights = self
    //         .insights_repo
    //         .read_range_by_instrument_id_and_feature_id(instrument_id, feature_id, from, to)
    //         .await?;

    //     let mut result = Vec::with_capacity(insights.len());
    //     for insight in insights {
    //         if let Some(id) = insight.instrument_id {
    //             let instrument = self.instrument_service.read_by_id(id).await?;
    //             result.push(Insight::new(
    //                 insight.event_time,
    //                 Some(instrument),
    //                 insight.feature_id,
    //                 insight.value,
    //             ));
    //         } else {
    //             result.push(Insight::new_general(insight.event_time, insight.feature_id, insight.value));
    //         }
    //     }
    //     Ok(result)
    // }
}
