use std::sync::Arc;

use arkin_core::{Strategy, StrategyListQuery, StrategyQuery};
use uuid::Uuid;

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

/// Load all strategies from database
pub async fn load_strategies(ctx: &PersistenceContext) -> Result<Vec<Arc<Strategy>>, PersistenceError> {
    let strategy_dtos = strategy_repo::list_all(ctx).await?;
    let mut strategies = Vec::with_capacity(strategy_dtos.len());

    for dto in strategy_dtos {
        let strategy: Arc<Strategy> = Arc::new(dto.into());
        update_cache(ctx, strategy.clone()).await;
        strategies.push(strategy);
    }

    Ok(strategies)
}

async fn update_cache(ctx: &PersistenceContext, strategy: Arc<Strategy>) {
    // Check if we already have this strategy cached
    if ctx.cache.strategy_id.get(&strategy.id).await.is_some() {
        return;
    }

    ctx.cache.strategy_id.insert(strategy.id, strategy).await;
}

/// Query strategies with in-memory filtering
pub async fn query(ctx: &PersistenceContext, query: &StrategyQuery) -> Result<Arc<Strategy>, PersistenceError> {
    // For now, load all and filter, since strategies are few
    let all_strategies = load_strategies(ctx).await?;
    for strategy in all_strategies {
        if query.matches(&strategy) {
            return Ok(strategy);
        }
    }
    Err(PersistenceError::NotFound)
}

/// Query strategies with in-memory filtering
pub async fn query_list(ctx: &PersistenceContext, query: &StrategyListQuery) -> Result<Vec<Arc<Strategy>>, PersistenceError> {
    let all_strategies = load_strategies(ctx).await?;

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_strategies);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Strategy>> = all_strategies.into_iter().filter(|strategy| query.matches(strategy)).collect();

    Ok(filtered)
}
