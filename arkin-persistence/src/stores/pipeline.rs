use std::sync::Arc;

use arkin_core::Pipeline;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{repos::PipelineRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]
pub struct PipelineStore {
    pipeline_repo: PipelineRepo,
}

impl PipelineStore {
    pub async fn insert(&self, pipeline: Arc<Pipeline>) -> Result<(), PersistenceError> {
        self.pipeline_repo.insert(pipeline.into()).await
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Pipeline>, PersistenceError> {
        let pipeline_dto = self.pipeline_repo.read_by_id(id).await?;
        let pipeline: Arc<Pipeline> = pipeline_dto.into();
        Ok(pipeline)
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Pipeline>, PersistenceError> {
        let pipeline_dto = self.pipeline_repo.read_by_name(name).await?;
        let pipeline: Arc<Pipeline> = pipeline_dto.into();
        Ok(pipeline)
    }
}
