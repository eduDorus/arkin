use std::sync::Arc;

use uuid::Uuid;

use arkin_core::Strategy;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::strategy_repo};

pub async fn insert(ctx: &PersistenceContext, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
    strategy_repo::insert(ctx, strategy.into()).await?;
    Ok(())
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

pub async fn update(ctx: &PersistenceContext, strategy: Arc<Strategy>) -> Result<(), PersistenceError> {
    strategy_repo::update(ctx, strategy.into()).await?;
    Ok(())
}

pub async fn delete(ctx: &PersistenceContext, id: &Uuid) -> Result<(), PersistenceError> {
    strategy_repo::delete(ctx, id).await?;
    Ok(())
}
