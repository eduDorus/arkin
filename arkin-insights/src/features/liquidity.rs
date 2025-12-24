use std::sync::Arc;

use async_trait::async_trait;
use time::UtcDateTime;

use arkin_core::prelude::*;

use crate::{
    config::LiquidityConfig,
    traits::{Feature, FillStrategy},
    FeatureStore, InstrumentScope,
};

#[derive(Debug)]
pub struct LiquidityFeature {
    pub config: LiquidityConfig,
    pub scopes: Vec<InstrumentScope>,
    pub output_ids: Vec<FeatureId>,
}

impl LiquidityFeature {
    pub fn new(config: LiquidityConfig, scopes: Vec<InstrumentScope>, output_ids: Vec<FeatureId>) -> Self {
        Self {
            config,
            scopes,
            output_ids,
        }
    }
}

#[async_trait]
impl Feature for LiquidityFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        // Liquidity features typically depend on raw trade data or basic metrics
        // For now, assume no inputs (or add if needed, e.g., price/volume features)
        vec![]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        self.output_ids.clone()
    }

    fn calculate(
        &self,
        state: &FeatureStore,
        _pipeline: &Arc<Pipeline>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        let mut insights = Vec::new();

        for scope in &self.scopes {
            // Aggregate data across instruments in scope
            let mut total_volume = 0.0;
            let mut spread_sum = 0.0;
            let mut depth_sum = 0.0;
            let mut count = 0;

            for instrument in &scope.inputs {
                // Fetch recent trades or metrics for this instrument
                // This is simplified; in reality, query state for rolling windows
                if let Some(volume) = state.last(instrument, &Arc::new("volume".to_string()), event_time) {
                    total_volume += volume;
                }
                if let Some(spread) = state.last(instrument, &Arc::new("spread".to_string()), event_time) {
                    spread_sum += spread;
                }
                if let Some(depth) = state.last(instrument, &Arc::new("depth".to_string()), event_time) {
                    depth_sum += depth;
                }
                count += 1;
            }

            if count > 0 {
                // Compute averages or totals
                let avg_spread = spread_sum / count as f64;
                let avg_depth = depth_sum / count as f64;

                // Create insights for each output
                for (i, output_name) in self.config.outputs.iter().enumerate() {
                    let value = match output_name.as_str() {
                        "total_volume" => total_volume,
                        "avg_spread" => avg_spread,
                        "avg_depth" => avg_depth,
                        _ => continue, // Skip unknown outputs
                    };

                    let insight = Arc::new(Insight {
                        event_time,
                        pipeline: Some(_pipeline.clone()),
                        instrument: scope.output.clone(), // Use output instrument
                        feature_id: self.output_ids[i].clone(),
                        value,
                        insight_type: InsightType::Raw,
                        persist: true,
                    });
                    insights.push(insight);
                }
            }
        }

        if insights.is_empty() {
            None
        } else {
            Some(insights)
        }
    }

    fn fill_strategy(&self) -> FillStrategy {
        self.config.fill_strategy
    }

    fn scopes(&self) -> &[InstrumentScope] {
        &self.scopes
    }
}