use std::sync::Arc;

use arkin_core::{Allocation, Insight, Pipeline, Strategy, Venue};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    stores::{AllocationStore, InsightsStore, PipelineStore, StrategyStore, VenueStore},
    PersistenceError,
};

#[derive(Debug, Clone, TypedBuilder)]

pub struct SystemService {
    pub pipeline_store: PipelineStore,
    pub venue_store: VenueStore,
    pub insight_store: InsightsStore,
    pub strategy_store: StrategyStore,
    pub allocation_store: AllocationStore,
}

impl SystemService {
    // Venues
    pub async fn insert_venue(&self, venue: Arc<Venue>) -> Result<(), PersistenceError> {
        self.venue_store.insert(venue).await
    }

    pub async fn read_venue_by_id(&self, id: &Uuid) -> Result<Arc<Venue>, PersistenceError> {
        self.venue_store.read_by_id(id).await
    }

    // Insights Pipeline
    pub async fn insert_pipeline(&self, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
        self.pipeline_store.insert(pipeline).await
    }

    pub async fn read_pipeline_by_id(&self, id: &Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        self.pipeline_store.read_by_id(id).await
    }

    pub async fn read_pipeline_by_name(&self, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
        self.pipeline_store.read_by_name(name).await
    }

    pub async fn insert_insight_buffered(&self, insight: Arc<Insight>) -> Result<(), PersistenceError> {
        self.insight_store.insert_buffered(insight).await
    }

    pub async fn insert_insight_buffered_vec(&self, insights: Vec<Arc<Insight>>) -> Result<(), PersistenceError> {
        self.insight_store.insert_buffered_vec(insights).await
    }

    // Strategy
    pub async fn insert_strategy(&self, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
        self.strategy_store.insert(strategy).await
    }

    pub async fn read_strategy_by_id(&self, id: &Uuid) -> Result<Arc<Strategy>, PersistenceError> {
        self.strategy_store.read_by_id(id).await
    }

    pub async fn read_strategy_by_name(&self, name: &str) -> Result<Arc<Strategy>, PersistenceError> {
        self.strategy_store.read_by_name(name).await
    }

    pub async fn update_strategy(&self, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
        self.strategy_store.update(strategy).await
    }

    pub async fn delete_strategy(&self, id: &Uuid) -> Result<(), PersistenceError> {
        self.strategy_store.delete(id).await
    }

    // Allocation
    pub async fn insert_allocation(&self, allocation: Arc<Allocation>) -> Result<(), PersistenceError> {
        self.allocation_store.insert(allocation).await
    }
}
