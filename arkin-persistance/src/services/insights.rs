use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;

use crate::repos::InsightsRepo;

#[derive(Debug)]
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
