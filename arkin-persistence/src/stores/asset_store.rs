use std::sync::Arc;

use uuid::Uuid;

use arkin_core::Asset;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::pg::asset_repo};

async fn update_cache(ctx: &PersistenceContext, asset: Arc<Asset>) {
    ctx.cache.asset_id.insert(asset.id, asset.clone()).await;
    ctx.cache.asset_symbol.insert(asset.name.clone(), asset).await;
}

async fn read_cache_by_id(ctx: &PersistenceContext, id: &Uuid) -> Option<Arc<Asset>> {
    ctx.cache.asset_id.get(id).await
}

async fn read_cache_by_symbol(ctx: &PersistenceContext, symbol: &str) -> Option<Arc<Asset>> {
    ctx.cache.asset_symbol.get(symbol).await
}

pub async fn insert(ctx: &PersistenceContext, asset: Arc<Asset>) -> Result<(), PersistenceError> {
    update_cache(ctx, asset.clone()).await;
    asset_repo::insert(ctx, asset.into()).await
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Asset>, PersistenceError> {
    if let Some(asset) = read_cache_by_id(ctx, id).await {
        return Ok(asset);
    }
    let asset_dto = asset_repo::read_by_id(ctx, id).await?;
    let asset: Arc<Asset> = asset_dto.into();
    update_cache(ctx, asset.clone()).await;
    Ok(asset)
}

pub async fn read_by_symbol(ctx: &PersistenceContext, symbol: &str) -> Result<Arc<Asset>, PersistenceError> {
    match read_cache_by_symbol(ctx, symbol).await {
        Some(asset) => return Ok(asset),
        None => {
            let asset_dto = asset_repo::read_by_symbol(ctx, symbol).await?;
            let asset: Arc<Asset> = asset_dto.into();
            update_cache(ctx, asset.clone()).await;
            Ok(asset)
        }
    }
}
