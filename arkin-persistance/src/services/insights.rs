use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use crate::repos::InsightsRepo;

use super::InstrumentService;

pub struct InsightsService {
    insights_repo: Arc<InsightsRepo>,
    instrument_service: Arc<InstrumentService>,
}

impl InsightsService {
    pub fn new(insights_repo: Arc<InsightsRepo>, instrument_service: Arc<InstrumentService>) -> Self {
        Self {
            insights_repo,
            instrument_service,
        }
    }

    pub async fn insert(&self, insight: Insight) -> Result<()> {
        self.insights_repo.insert(insight).await
    }

    pub async fn insert_batch(&self, insights: Vec<Insight>) -> Result<()> {
        self.insights_repo.insert_batch(insights).await
    }

    pub async fn read_range_by_instrument_id_and_feature_id(
        &self,
        instrument_id: &Uuid,
        feature_id: &str,
        from: &OffsetDateTime,
        to: &OffsetDateTime,
    ) -> Result<Vec<Insight>> {
        // Load insights
        let insights = self
            .insights_repo
            .read_range_by_instrument_id_and_feature_id(instrument_id, feature_id, from, to)
            .await?;

        let mut result = Vec::with_capacity(insights.len());
        for insight in insights {
            if let Ok(instrument) = self.instrument_service.read_by_id(&insight.instrument_id).await {
                if let Some(instrument) = instrument {
                    result.push(Insight::new(insight.event_time, instrument, insight.feature_id, insight.value));
                } else {
                    error!("Instrument not found: {}", insight.instrument_id);
                }
            } else {
                error!("Could not fetch instrument: {}", insight.instrument_id);
            }
        }

        Ok(result)
    }
}
