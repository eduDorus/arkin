use std::sync::Arc;

use uuid::Uuid;

use arkin_core::{Asset, AssetQuery};

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

/// Load all assets from database into cache
/// This should be called once during persistence initialization
pub async fn load_assets(ctx: &PersistenceContext) -> Result<Vec<Arc<Asset>>, PersistenceError> {
    let asset_dtos = asset_repo::list_all(ctx).await?;
    let mut assets = Vec::with_capacity(asset_dtos.len());

    for dto in asset_dtos {
        let asset: Arc<Asset> = dto.into();
        update_cache(ctx, asset.clone()).await;
        assets.push(asset);
    }

    Ok(assets)
}

/// Query assets with in-memory filtering from cache
/// Assumes cache is already populated via load_assets()
pub async fn query(ctx: &PersistenceContext, query: &AssetQuery) -> Result<Vec<Arc<Asset>>, PersistenceError> {
    // Get all cached assets by iterating over the cache
    let all_assets: Vec<Arc<Asset>> = ctx.cache.asset_id.iter().map(|(_, asset)| asset).collect();

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_assets);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Asset>> = all_assets.into_iter().filter(|asset| query.matches(asset)).collect();

    Ok(filtered)
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
        Some(asset) => Ok(asset),
        None => {
            let asset_dto = asset_repo::read_by_symbol(ctx, symbol).await?;
            let asset: Arc<Asset> = asset_dto.into();
            update_cache(ctx, asset.clone()).await;
            Ok(asset)
        }
    }
}
