use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;

use crate::repos::InsightsRepo;

pub struct InsightsService {
    insights_repo: Arc<InsightsRepo>,
}

impl InsightsService {
    pub fn new(insights_repo: Arc<InsightsRepo>) -> Self {
        Self { insights_repo }
    }

    pub async fn insert(&self, insight: Insight) -> Result<()> {
        self.insights_repo.insert(insight).await
    }

    pub async fn insert_batch(&self, insights: Vec<Insight>) -> Result<()> {
        self.insights_repo.insert_batch(insights).await
    }
}
