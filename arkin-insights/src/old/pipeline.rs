use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

use arkin_core::{Insight, Instrument, Pipeline};
use time::UtcDateTime;
use tracing::info;

use crate::{config::PipelineConfig, state::InsightsState, CompGraph, Feature};

/// Unified Pipeline combining state management and computation graph
///
/// This wrapper provides a clean API that:
/// - Automatically filters incoming insights to raw inputs only
/// - Manages state commits
/// - Executes the computation graph
/// - Tracks warmup status to ensure sufficient data before producing results
pub struct FeaturePipeline {
    meta: Arc<Pipeline>, // Metadata about the pipeline (from arkin-core)
    warmup_steps: AtomicU16,
    state: InsightsState,
    graph: CompGraph,
}

impl FeaturePipeline {
    pub fn new(pipeline_meta: Arc<Pipeline>, features: Vec<Arc<dyn Feature>>, config: &PipelineConfig) -> Self {
        Self {
            meta: pipeline_meta,
            warmup_steps: AtomicU16::new(config.warmup_steps),
            state: InsightsState::new(config.state_ttl),
            graph: CompGraph::new(features),
        }
    }

    /// Check if the pipeline has completed its warmup period
    pub fn is_ready(&self) -> bool {
        self.warmup_steps.load(Ordering::Relaxed) == 0
    }

    /// Get the number of remaining warmup steps
    pub fn warmup_remaining(&self) -> u16 {
        self.warmup_steps.load(Ordering::Relaxed)
    }

    pub async fn insert(&self, insight: Arc<Insight>) {
        let raw_inputs = self.graph.get_raw_inputs();

        // Only insert if it's a raw input
        if raw_inputs.contains(&insight.feature_id) {
            self.state.insert_buffered(insight).await;
        }
    }

    /// Insert insights, automatically filtering to only raw inputs (those not produced by any feature)
    pub async fn insert_batch(&self, insights: &[Arc<Insight>]) {
        let raw_inputs = self.graph.get_raw_inputs();

        // Filter insights to only raw inputs
        let filtered: Vec<Arc<Insight>> = insights
            .iter()
            .filter(|insight| raw_inputs.contains(&insight.feature_id))
            .cloned()
            .collect();

        if !filtered.is_empty() {
            self.state.insert_batch_buffered(&filtered).await;
        }
    }

    /// Commit buffered data to state
    pub async fn commit(&self, event_time: UtcDateTime) {
        self.state.commit(event_time).await;
    }

    /// Calculate all features for given instruments
    ///
    /// During warmup period, features are still calculated to build up derived features,
    /// but an empty vector is returned. Once warmup is complete, calculated insights are returned.
    pub fn calculate(&self, event_time: UtcDateTime, instruments: &[Arc<Instrument>]) -> Vec<Arc<Insight>> {
        // Always calculate to build up feature dependencies, even during warmup
        let insights = self.graph.calculate(&self.state, &self.meta, event_time, instruments);

        // Decrement warmup counter if still in warmup
        let remaining = self.warmup_steps.load(Ordering::Relaxed);
        if remaining > 0 {
            let new_remaining = self.warmup_steps.fetch_sub(1, Ordering::Relaxed) - 1;
            info!(
                target: "insights",
                "warmup tick at {}, {} remaining",
                event_time,
                new_remaining
            );
            // Return empty vec during warmup - features are calculated but not published
            Vec::new()
        } else {
            // Warmup complete, return calculated insights
            insights
        }
    }

    /// Get reference to underlying state
    pub fn state(&self) -> &InsightsState {
        &self.state
    }

    /// Get reference to underlying computation graph
    pub fn graph(&self) -> &CompGraph {
        &self.graph
    }

    /// Get reference to pipeline metadata
    pub fn pipeline_meta(&self) -> &Arc<Pipeline> {
        &self.meta
    }
}
