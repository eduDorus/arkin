use std::sync::Arc;

use uuid::Uuid;

use arkin_core::Strategy;

use crate::{context::PersistenceContext, repos::pg::strategy_repo, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
    strategy_repo::insert(ctx, strategy.into()).await
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Strategy>, PersistenceError> {
    let strategy_dto = strategy_repo::read_by_id(ctx, id).await?;
    let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
    Ok(strategy)
}

pub async fn read_by_name(ctx: &PersistenceContext, name: &str) -> Result<Arc<Strategy>, PersistenceError> {
    let strategy_dto = strategy_repo::read_by_name(ctx, name).await?;
    let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
    Ok(strategy)
}

pub async fn read_by_name_or_create(ctx: &PersistenceContext, name: &str) -> Result<Arc<Strategy>, PersistenceError> {
    if let Ok(strategy_dto) = strategy_repo::read_by_name(ctx, name).await {
        let strategy: Arc<Strategy> = Arc::new(Strategy::from(strategy_dto));
        return Ok(strategy);
    } else {
        let strategy = Arc::new(Strategy::builder().name(name.to_string()).description(None).build());
        strategy_repo::insert(ctx, strategy.clone().into()).await?;
        return Ok(strategy);
    }
}

pub async fn update(ctx: &PersistenceContext, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
    strategy_repo::update(ctx, strategy.into()).await
}

pub async fn delete(ctx: &PersistenceContext, id: &Uuid) -> Result<(), PersistenceError> {
    strategy_repo::delete(ctx, id).await
}
