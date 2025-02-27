use std::sync::Arc;

use arkin_core::Strategy;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{repos::StrategyRepo, PersistenceError};

#[derive(Debug, Clone, TypedBuilder)]
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

    pub async fn read_by_name_or_create(&self, name: &str) -> Result<Arc<Strategy>, PersistenceError> {
        if let Ok(strategy_dto) = self.strategy_repo.read_by_name(name).await {
            let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
            return Ok(strategy);
        } else {
            let strategy = Arc::new(Strategy::builder().name(name.to_string()).description(None).build());
            self.insert(strategy.clone()).await?;
            return Ok(strategy);
        }
    }

    pub async fn update(&self, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
        self.strategy_repo.update(strategy.into()).await
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), PersistenceError> {
        self.strategy_repo.delete(id).await
    }
}
