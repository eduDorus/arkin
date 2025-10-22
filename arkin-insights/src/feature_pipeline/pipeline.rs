use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};
use std::time::Duration;

use arkin_core::{Insight, Instrument, Pipeline};
use time::UtcDateTime;
use tracing::info;

use crate::{config::PipelineConfig, Feature, FeatureGraph, FeatureStore};

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
    state: Arc<FeatureStore>,
    graph: Arc<FeatureGraph>,
}

impl FeaturePipeline {
    pub fn new(pipeline_meta: Arc<Pipeline>, features: Vec<Arc<dyn Feature>>, config: &PipelineConfig) -> Self {
        Self {
            meta: pipeline_meta,
            warmup_steps: AtomicU16::new(config.warmup_steps),
            state: Arc::new(
                FeatureStore::builder()
                    .ttl(Duration::from_secs(config.state_ttl))
                    .min_interval(config.min_interval)
                    .build(),
            ),
            graph: Arc::new(FeatureGraph::new(features, config.parallel)),
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
    pub async fn calculate(&self, event_time: UtcDateTime, instruments: &[Arc<Instrument>]) -> Vec<Arc<Insight>> {
        // Always calculate to build up feature dependencies, even during warmup
        let insights = tokio::task::spawn_blocking({
            let graph = Arc::clone(&self.graph);
            let state = Arc::clone(&self.state); // If FeatureState needs Arc
            let meta = Arc::clone(&self.meta);
            let instruments = instruments.to_vec(); // Clone slice
            let event_time = event_time;
            move || graph.calculate(&state, &meta, event_time, &instruments)
        })
        .await
        .expect("Failed to calculate pipeline");

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
    pub fn state(&self) -> &FeatureStore {
        &self.state
    }

    /// Get reference to underlying computation graph
    pub fn graph(&self) -> &FeatureGraph {
        &self.graph
    }

    /// Get reference to pipeline metadata
    pub fn pipeline_meta(&self) -> &Arc<Pipeline> {
        &self.meta
    }
}

#[cfg(test)]
mod tests {
    use crate::FillStrategy;

    use super::*;
    use arkin_core::test_utils::{test_inst_binance_btc_usdt_perp, test_inst_binance_eth_usdt_perp, test_pipeline};
    use arkin_core::{FeatureId, InsightType};
    use time::Duration;

    fn create_test_insight(
        pipeline: Arc<Pipeline>,
        instrument: Arc<Instrument>,
        feature_id: FeatureId,
        event_time: UtcDateTime,
        value: f64,
    ) -> Arc<Insight> {
        Arc::new(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(pipeline))
                .instrument(instrument)
                .feature_id(feature_id)
                .value(value)
                .insight_type(InsightType::Raw)
                .build(),
        )
    }

    fn create_test_config(warmup_steps: u16) -> PipelineConfig {
        PipelineConfig {
            version: "test".to_string(),
            warmup_steps,
            state_ttl: Duration::hours(24).whole_seconds() as u64,
            min_interval: 60,
            features: Vec::new(),
            parallel: false,
        }
    }

    #[tokio::test]
    async fn test_warmup_countdown() {
        let pipeline_meta = test_pipeline();
        let features: Vec<Arc<dyn Feature>> = Vec::new();
        let config = create_test_config(5);

        let pipeline = FeaturePipeline::new(pipeline_meta, features, &config);

        // Initial warmup should be 5
        assert_eq!(pipeline.warmup_remaining(), 5);
        assert!(!pipeline.is_ready());

        let now = UtcDateTime::now();
        let instruments = vec![test_inst_binance_btc_usdt_perp()];

        // First calculate - warmup should decrement to 4
        let results = pipeline.calculate(now, &instruments).await;
        assert_eq!(results.len(), 0); // Should return empty during warmup
        assert_eq!(pipeline.warmup_remaining(), 4);
        assert!(!pipeline.is_ready());

        // Second calculate - warmup should decrement to 3
        let results = pipeline.calculate(now + Duration::seconds(1), &instruments).await;
        assert_eq!(results.len(), 0);
        assert_eq!(pipeline.warmup_remaining(), 3);

        // Third calculate - warmup should decrement to 2
        let results = pipeline.calculate(now + Duration::seconds(2), &instruments).await;
        assert_eq!(results.len(), 0);
        assert_eq!(pipeline.warmup_remaining(), 2);

        // Fourth calculate - warmup should decrement to 1
        let results = pipeline.calculate(now + Duration::seconds(3), &instruments).await;
        assert_eq!(results.len(), 0);
        assert_eq!(pipeline.warmup_remaining(), 1);

        // Fifth calculate - warmup should decrement to 0
        let results = pipeline.calculate(now + Duration::seconds(4), &instruments).await;
        assert_eq!(results.len(), 0);
        assert_eq!(pipeline.warmup_remaining(), 0);
        assert!(pipeline.is_ready());

        // Sixth calculate - warmup complete, should return results (empty because no features)
        let results = pipeline.calculate(now + Duration::seconds(5), &instruments).await;
        assert_eq!(pipeline.warmup_remaining(), 0);
        assert!(pipeline.is_ready());
        // Results are empty because we have no features configured
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_warmup_is_ready() {
        let pipeline_meta = test_pipeline();
        let features: Vec<Arc<dyn Feature>> = Vec::new();

        // Test with 0 warmup
        let config_no_warmup = create_test_config(0);
        let pipeline_no_warmup = FeaturePipeline::new(pipeline_meta.clone(), features.clone(), &config_no_warmup);
        assert_eq!(pipeline_no_warmup.warmup_remaining(), 0);
        assert!(pipeline_no_warmup.is_ready());

        // Test with warmup
        let config_warmup = create_test_config(3);
        let pipeline_warmup = FeaturePipeline::new(pipeline_meta, features, &config_warmup);
        assert_eq!(pipeline_warmup.warmup_remaining(), 3);
        assert!(!pipeline_warmup.is_ready());

        // After one calculate, still not ready
        let now = UtcDateTime::now();
        let instruments = vec![test_inst_binance_btc_usdt_perp()];
        pipeline_warmup.calculate(now, &instruments).await;
        assert!(!pipeline_warmup.is_ready());

        // After three calculates, should be ready
        pipeline_warmup.calculate(now + Duration::seconds(1), &instruments).await;
        pipeline_warmup.calculate(now + Duration::seconds(2), &instruments).await;
        assert!(pipeline_warmup.is_ready());
    }

    #[tokio::test]
    async fn test_insert_filtering_raw_inputs() {
        use crate::features::{LagAlgo, LagFeature};

        let pipeline_meta = test_pipeline();
        let instrument = test_inst_binance_btc_usdt_perp();
        let raw_feature_id = FeatureId::new("close".to_string());
        let derived_feature_id = FeatureId::new("close_log_change".to_string());

        // Create a lag feature that produces derived_feature_id from raw_feature_id
        let lag_feature = LagFeature::builder()
            .input(raw_feature_id.clone())
            .output(derived_feature_id.clone())
            .lag(1)
            .method(LagAlgo::LogChange)
            .fill_strategy(FillStrategy::ForwardFill)
            .persist(false)
            .build();
        let features: Vec<Arc<dyn Feature>> = vec![Arc::new(lag_feature)];

        let config = create_test_config(0);
        let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config);

        let now = UtcDateTime::now();

        // Insert a raw input insight (should be accepted)
        let raw_insight =
            create_test_insight(pipeline_meta.clone(), instrument.clone(), raw_feature_id.clone(), now, 100.0);
        pipeline.insert(raw_insight).await;

        // Insert a derived insight (should be filtered out)
        let derived_insight =
            create_test_insight(pipeline_meta.clone(), instrument.clone(), derived_feature_id.clone(), now, 0.05);
        pipeline.insert(derived_insight).await;

        // Commit to move from buffer to state
        pipeline.commit(now).await;

        // Only the raw input should be in state
        let raw_value = pipeline.state().last(&instrument, &raw_feature_id, now);
        assert_eq!(raw_value, Some(100.0));

        // The derived feature should NOT be in state (was filtered)
        let derived_value = pipeline.state().last(&instrument, &derived_feature_id, now);
        assert_eq!(derived_value, None);
    }

    #[tokio::test]
    async fn test_insert_batch_filtering() {
        use crate::features::{LagAlgo, LagFeature};

        let pipeline_meta = test_pipeline();
        let instrument = test_inst_binance_btc_usdt_perp();
        let raw_feature_id = FeatureId::new("close".to_string());
        let derived_feature_id = FeatureId::new("close_log_change".to_string());

        let lag_feature = LagFeature::builder()
            .input(raw_feature_id.clone())
            .output(derived_feature_id.clone())
            .lag(1)
            .method(LagAlgo::LogChange)
            .fill_strategy(FillStrategy::ForwardFill)
            .persist(false)
            .build();
        let features: Vec<Arc<dyn Feature>> = vec![Arc::new(lag_feature)];

        let config = create_test_config(0);
        let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config);

        let now = UtcDateTime::now();

        // Create batch with mixed raw and derived insights at grid-aligned timestamps
        let insights = vec![
            create_test_insight(pipeline_meta.clone(), instrument.clone(), raw_feature_id.clone(), now, 100.0),
            create_test_insight(pipeline_meta.clone(), instrument.clone(), derived_feature_id.clone(), now, 0.05),
            create_test_insight(
                pipeline_meta.clone(),
                instrument.clone(),
                raw_feature_id.clone(),
                now + Duration::seconds(60), // Grid-aligned: 60s later
                105.0,
            ),
        ];

        pipeline.insert_batch(&insights).await;
        pipeline.commit(now + Duration::seconds(60)).await;

        // Only raw inputs should be in state
        // Query last 2 intervals: [now, now+60]
        let values = pipeline
            .state()
            .interval(&instrument, &raw_feature_id, now + Duration::seconds(60), 2, None)
            .unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], 100.0);
        assert_eq!(values[1], 105.0);

        // Derived feature should not be in state
        let derived_value = pipeline
            .state()
            .last(&instrument, &derived_feature_id, now + Duration::seconds(60));
        assert_eq!(derived_value, None);
    }

    #[tokio::test]
    async fn test_state_and_graph_accessors() {
        let pipeline_meta = test_pipeline();
        let features: Vec<Arc<dyn Feature>> = Vec::new();
        let config = create_test_config(0);

        let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config);

        // Test accessors
        assert!(Arc::ptr_eq(pipeline.pipeline_meta(), &pipeline_meta));
        assert!(pipeline.graph().get_raw_inputs().is_empty());
    }

    #[tokio::test]
    async fn test_multiple_instruments() {
        use crate::features::{LagAlgo, LagFeature};

        let pipeline_meta = test_pipeline();
        let raw_feature_id = FeatureId::new("close".to_string());
        let derived_feature_id = FeatureId::new("close_log_change".to_string());

        // Create a feature that depends on "close" so it becomes a raw input
        let lag_feature = LagFeature::builder()
            .input(raw_feature_id.clone())
            .output(derived_feature_id)
            .lag(1)
            .method(LagAlgo::LogChange)
            .fill_strategy(FillStrategy::ForwardFill)
            .persist(false)
            .build();
        let features: Vec<Arc<dyn Feature>> = vec![Arc::new(lag_feature)];
        let config = create_test_config(0);

        let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config);

        let now = UtcDateTime::now();
        let btc = test_inst_binance_btc_usdt_perp();
        let eth = test_inst_binance_eth_usdt_perp();

        // Insert insights for both instruments
        let btc_insight = create_test_insight(pipeline_meta.clone(), btc.clone(), raw_feature_id.clone(), now, 50000.0);
        let eth_insight = create_test_insight(pipeline_meta.clone(), eth.clone(), raw_feature_id.clone(), now, 3000.0);

        pipeline.insert(btc_insight).await;
        pipeline.insert(eth_insight).await;
        pipeline.commit(now).await;

        // Verify both instruments have their data
        let btc_value = pipeline.state().last(&btc, &raw_feature_id, now);
        let eth_value = pipeline.state().last(&eth, &raw_feature_id, now);

        assert_eq!(btc_value, Some(50000.0));
        assert_eq!(eth_value, Some(3000.0));
    }
}
