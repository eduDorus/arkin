use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;

pub struct InsightsFactory {}

impl InsightsFactory {
    pub async fn init(
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
        pipeline: &str,
    ) -> Arc<dyn RunnableService> {
        let config = load::<InsightsConfig>();

        // Load pipeline
        let pipeline = persistence
            .pipeline_store
            .read_by_name(&pipeline)
            .await
            .expect("Pipeline not found");

        let insights: Arc<dyn RunnableService> =
            InsightsService::init(pubsub.clone(), pipeline, &config.insights_service.pipeline).await;
        insights
    }
}
