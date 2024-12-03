use std::sync::Arc;

use arkin_core::Strategy;
use derive_builder::Builder;
use uuid::Uuid;

use crate::{repos::StrategyRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct StrategyStore {
    strategy_repo: StrategyRepo,
}

impl StrategyStore {
    pub async fn insert(&self, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
        self.strategy_repo.insert(strategy.into()).await
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Strategy>, PersistenceError> {
        let strategy_dto = self.strategy_repo.read_by_id(id).await?;
        let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
        Ok(strategy)
    }

    pub async fn read_by_name(&self, name: &str) -> Result<Arc<Strategy>, PersistenceError> {
        let strategy_dto = self.strategy_repo.read_by_name(name).await?;
        let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
        Ok(strategy)
    }

    pub async fn update(&self, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
        self.strategy_repo.update(strategy.into()).await
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
        self.strategy_repo.delete(id).await
    }
}
